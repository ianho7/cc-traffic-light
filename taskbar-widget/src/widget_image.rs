//! Agent logo rendering for the taskbar widget.
//!
//! Logos are embedded from PNG files via `include_bytes!`
//! (resources/logos/codex.png, claude.png) and decoded at first use
//! via the `image` crate.

use std::{
    collections::HashMap,
    fs,
    sync::{Arc, Mutex, OnceLock},
    time::SystemTime,
};

use image::{imageops::FilterType, load_from_memory};

use crate::{
    app_config::MaterialGroup,
    widget_render::{PixelBuffer, Rgba, WidgetGroupId},
};

/// Decoded RGBA logo data ready for pixel-blit rendering.
pub struct LogoData {
    pub width: u32,
    pub height: u32,
    pub pixels: &'static [u8],
}

/// Decoded, taskbar-sized image data for one user-provided material slot.
#[derive(Clone, Debug)]
pub struct MaterialImageData {
    pub width: u32,
    pub height: u32,
    pub pixels: Vec<u8>,
}

/// The fixed green/yellow/red material set used by one widget group.
#[derive(Clone, Debug)]
pub struct MaterialGroupImages {
    pub green: MaterialImageData,
    pub yellow: MaterialImageData,
    pub red: MaterialImageData,
}

#[derive(Clone, Debug, Eq, PartialEq)]
struct MaterialFileStamp {
    path: String,
    modified: SystemTime,
    len: u64,
}

#[derive(Clone, Debug, Eq, PartialEq)]
struct MaterialGroupStamp {
    green: MaterialFileStamp,
    yellow: MaterialFileStamp,
    red: MaterialFileStamp,
}

struct CachedMaterialGroup {
    stamp: MaterialGroupStamp,
    images: Arc<MaterialGroupImages>,
}

static MATERIAL_GROUP_CACHE: OnceLock<Mutex<HashMap<(String, u32), CachedMaterialGroup>>> =
    OnceLock::new();

fn decode_logo(bytes: &[u8]) -> &'static LogoData {
    let img = load_from_memory(bytes).expect("Failed to decode logo PNG");
    let rgba = img.to_rgba8();
    let (width, height) = rgba.dimensions();
    let pixels: &'static [u8] = Box::leak(rgba.into_raw().into_boxed_slice());
    // Leak the LogoData itself so it lives for the program's lifetime.
    let logo: &'static LogoData = Box::leak(Box::new(LogoData {
        width,
        height,
        pixels,
    }));
    logo
}

/// Return the logo data for the given group ID.
pub fn get_logo(id: WidgetGroupId) -> &'static LogoData {
    match id {
        WidgetGroupId::Codex => {
            static LOGO: OnceLock<&LogoData> = OnceLock::new();
            *LOGO.get_or_init(|| decode_logo(include_bytes!("../resources/codex.png")))
        }
        WidgetGroupId::Claude => {
            static LOGO: OnceLock<&LogoData> = OnceLock::new();
            *LOGO.get_or_init(|| decode_logo(include_bytes!("../resources/claude.png")))
        }
    }
}

/// Load a material group from its three local PNG files.
///
/// Cached decoded pixels are reused until one of the source files changes.
/// Returning an error intentionally makes the caller fall back to the built-in
/// lamps, so a missing or corrupt user asset never breaks the widget.
pub fn get_material_group_images(
    group: &MaterialGroup,
    size: u32,
) -> Result<Arc<MaterialGroupImages>, String> {
    let stamp = material_group_stamp(group)?;
    let cache = MATERIAL_GROUP_CACHE.get_or_init(|| Mutex::new(HashMap::new()));
    let mut cache = cache
        .lock()
        .map_err(|_| "material image cache lock poisoned".to_string())?;
    let cache_key = (group.id.clone(), size);

    if let Some(cached) = cache.get(&cache_key) {
        if cached.stamp == stamp {
            return Ok(Arc::clone(&cached.images));
        }
    }

    let images = Arc::new(MaterialGroupImages {
        green: decode_material_image(&group.green_path, size)?,
        yellow: decode_material_image(&group.yellow_path, size)?,
        red: decode_material_image(&group.red_path, size)?,
    });
    cache.insert(
        cache_key,
        CachedMaterialGroup {
            stamp,
            images: Arc::clone(&images),
        },
    );
    Ok(images)
}

fn material_group_stamp(group: &MaterialGroup) -> Result<MaterialGroupStamp, String> {
    Ok(MaterialGroupStamp {
        green: material_file_stamp(&group.green_path)?,
        yellow: material_file_stamp(&group.yellow_path)?,
        red: material_file_stamp(&group.red_path)?,
    })
}

fn material_file_stamp(path: &str) -> Result<MaterialFileStamp, String> {
    let metadata =
        fs::metadata(path).map_err(|error| format!("cannot read material {path}: {error}"))?;
    let modified = metadata
        .modified()
        .map_err(|error| format!("cannot inspect material {path}: {error}"))?;
    Ok(MaterialFileStamp {
        path: path.to_string(),
        modified,
        len: metadata.len(),
    })
}

fn decode_material_image(path: &str, size: u32) -> Result<MaterialImageData, String> {
    let bytes = fs::read(path).map_err(|error| format!("cannot read material {path}: {error}"))?;
    let image = load_from_memory(&bytes)
        .map_err(|error| format!("cannot decode material {path}: {error}"))?
        .to_rgba8();
    let image = image::imageops::resize(&image, size, size, FilterType::Triangle);
    let (width, height) = image.dimensions();
    Ok(MaterialImageData {
        width,
        height,
        pixels: image.into_raw(),
    })
}

/// Blit a logo onto the pixel buffer at the given top-left position.
///
/// Pixels with alpha = 0 are skipped; other pixels are alpha-blended onto the
/// buffer using its existing `blend_pixel` logic.
pub(crate) fn blit_logo(buffer: &mut PixelBuffer, left: i32, top: i32, logo: &LogoData) {
    let w = logo.width as i32;
    let h = logo.height as i32;
    let buf_w = buffer.width();
    let buf_h = buffer.height();

    let src_x_start = 0.max(-left);
    let src_y_start = 0.max(-top);
    let src_x_end = w.min(buf_w - left);
    let src_y_end = h.min(buf_h - top);

    for sy in src_y_start..src_y_end {
        for sx in src_x_start..src_x_end {
            let src_idx = ((sy as usize) * logo.width as usize + (sx as usize)) * 4;
            let alpha = logo.pixels[src_idx + 3];
            if alpha == 0 {
                continue;
            }
            let color = Rgba {
                red: logo.pixels[src_idx],
                green: logo.pixels[src_idx + 1],
                blue: logo.pixels[src_idx + 2],
                alpha,
            };
            buffer.blend_pixel(left + sx, top + sy, color);
        }
    }
}

/// Alpha-blend a taskbar material image into the existing pixel buffer.
pub(crate) fn blit_material(
    buffer: &mut PixelBuffer,
    left: i32,
    top: i32,
    image: &MaterialImageData,
    opacity: u8,
) {
    let w = image.width as i32;
    let h = image.height as i32;
    let buf_w = buffer.width();
    let buf_h = buffer.height();
    let src_x_start = 0.max(-left);
    let src_y_start = 0.max(-top);
    let src_x_end = w.min(buf_w - left);
    let src_y_end = h.min(buf_h - top);

    for sy in src_y_start..src_y_end {
        for sx in src_x_start..src_x_end {
            let src_idx = ((sy as usize * image.width as usize) + sx as usize) * 4;
            let alpha = ((u16::from(image.pixels[src_idx + 3]) * u16::from(opacity)) / 255) as u8;
            if alpha == 0 {
                continue;
            }
            buffer.blend_pixel(
                left + sx,
                top + sy,
                Rgba {
                    red: image.pixels[src_idx],
                    green: image.pixels[src_idx + 1],
                    blue: image.pixels[src_idx + 2],
                    alpha,
                },
            );
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::{env, fs, path::Path};

    use image::{ImageBuffer, RgbaImage};

    #[test]
    fn codex_logo_decodes_and_has_visible_pixels() {
        let logo = get_logo(WidgetGroupId::Codex);
        assert!(logo.width > 0, "codex logo should have positive width");
        assert!(logo.height > 0, "codex logo should have positive height");
        assert!(logo.pixels.len() == (logo.width * logo.height * 4) as usize);

        let non_zero = logo.pixels.iter().any(|&b| b != 0);
        assert!(non_zero, "codex logo should have non-zero pixel data");
    }

    #[test]
    fn claude_logo_decodes_and_has_visible_pixels() {
        let logo = get_logo(WidgetGroupId::Claude);
        assert!(logo.width > 0);
        assert!(logo.height > 0);
        let non_zero = logo.pixels.iter().any(|&b| b != 0);
        assert!(non_zero, "claude logo should have non-zero pixel data");
    }

    fn write_test_png(path: &Path, width: u32, color: [u8; 4]) {
        let image: RgbaImage = ImageBuffer::from_pixel(width, 4, image::Rgba(color));
        image.save(path).expect("test PNG should save");
    }

    fn test_group(root: &Path) -> MaterialGroup {
        MaterialGroup {
            id: format!("test-{}", std::process::id()),
            name: "Test".to_string(),
            green_path: root.join("green.png").display().to_string(),
            yellow_path: root.join("yellow.png").display().to_string(),
            red_path: root.join("red.png").display().to_string(),
        }
    }

    #[test]
    fn material_group_decodes_to_requested_size_and_reloads_when_changed() {
        let root =
            env::temp_dir().join(format!("cc-traffic-light-material-{}", std::process::id()));
        let _ = fs::remove_dir_all(&root);
        fs::create_dir_all(&root).expect("test directory should exist");
        for (name, color) in [
            ("green.png", [0, 255, 0, 255]),
            ("yellow.png", [255, 255, 0, 255]),
            ("red.png", [255, 0, 0, 255]),
        ] {
            write_test_png(&root.join(name), 4, color);
        }
        let group = test_group(&root);

        let first = get_material_group_images(&group, 16).expect("material group should decode");
        assert_eq!((first.green.width, first.green.height), (16, 16));
        assert_eq!(&first.green.pixels[..4], &[0, 255, 0, 255]);

        write_test_png(&root.join("green.png"), 5, [0, 0, 255, 255]);
        let second =
            get_material_group_images(&group, 16).expect("changed material group should decode");
        assert_eq!(&second.green.pixels[..4], &[0, 0, 255, 255]);

        let enlarged = get_material_group_images(&group, 32)
            .expect("different requested size should decode a new cache entry");
        assert_eq!((enlarged.green.width, enlarged.green.height), (32, 32));

        let _ = fs::remove_dir_all(root);
    }

    #[test]
    fn missing_material_image_returns_error() {
        let group = MaterialGroup {
            id: "missing".to_string(),
            name: "Missing".to_string(),
            green_path: "C:/does-not-exist/green.png".to_string(),
            yellow_path: "C:/does-not-exist/yellow.png".to_string(),
            red_path: "C:/does-not-exist/red.png".to_string(),
        };

        assert!(get_material_group_images(&group, 16).is_err());
    }
}
