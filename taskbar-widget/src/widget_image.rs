//! Agent logo rendering for the taskbar widget.
//!
//! Logos are embedded from PNG files via `include_bytes!`
//! (resources/logos/codex.png, claude.png) and decoded at first use
//! via the `image` crate.

use std::sync::OnceLock;

use image::load_from_memory;

use crate::widget_render::{PixelBuffer, Rgba, WidgetGroupId};

/// Decoded RGBA logo data ready for pixel-blit rendering.
pub struct LogoData {
    pub width: u32,
    pub height: u32,
    pub pixels: &'static [u8],
}

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
            *LOGO.get_or_init(|| {
                decode_logo(include_bytes!("../resources/logos/codex.png"))
            })
        }
        WidgetGroupId::Claude => {
            static LOGO: OnceLock<&LogoData> = OnceLock::new();
            *LOGO.get_or_init(|| {
                decode_logo(include_bytes!("../resources/logos/claude.png"))
            })
        }
    }
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

#[cfg(test)]
mod tests {
    use super::*;

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
}
