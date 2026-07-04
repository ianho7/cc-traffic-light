use crate::{
    app_config::{AppConfig, WidgetPaletteConfig},
    ui_state::{AppStatusSnapshot, SourceVisualState},
};
use windows::Win32::{
    Foundation::{COLORREF, HWND, POINT, RECT, SIZE},
    Graphics::Gdi::{
        AC_SRC_ALPHA, AC_SRC_OVER, BI_RGB, BITMAPINFO, BITMAPINFOHEADER, BLENDFUNCTION,
        CreateCompatibleDC, CreateDIBSection, DIB_RGB_COLORS, DeleteDC, DeleteObject, GetDC,
        HGDIOBJ, RGBQUAD, ReleaseDC, SelectObject,
    },
    UI::WindowsAndMessaging::{ULW_ALPHA, UpdateLayeredWindow},
};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum WidgetGroupId {
    Codex,
    Claude,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct WidgetHotZone {
    pub group: WidgetGroupId,
    pub rect: RECT,
}

#[derive(Clone, Debug)]
pub struct WidgetFrame {
    pub hot_zones: Vec<WidgetHotZone>,
    width: i32,
    height: i32,
    pixels: Vec<u8>,
}

#[derive(Clone, Copy)]
struct Rgba {
    red: u8,
    green: u8,
    blue: u8,
    alpha: u8,
}

struct PixelBuffer {
    width: i32,
    height: i32,
    pixels: Vec<u8>,
}

struct GroupLayout {
    id: WidgetGroupId,
    label: char,
    state: SourceVisualState,
    hot_zone: RECT,
    lights: [LampLayout; 3],
}

#[derive(Clone, Copy)]
struct LampLayout {
    center_x: f32,
    center_y: f32,
    radius: f32,
}

#[derive(Clone, Copy)]
struct Palette {
    green: Rgba,
    yellow: Rgba,
    red: Rgba,
    off: Rgba,
}

const SUPERSAMPLE_GRID: [(f32, f32); 16] = [
    (0.125, 0.125),
    (0.375, 0.125),
    (0.625, 0.125),
    (0.875, 0.125),
    (0.125, 0.375),
    (0.375, 0.375),
    (0.625, 0.375),
    (0.875, 0.375),
    (0.125, 0.625),
    (0.375, 0.625),
    (0.625, 0.625),
    (0.875, 0.625),
    (0.125, 0.875),
    (0.375, 0.875),
    (0.625, 0.875),
    (0.875, 0.875),
];

pub fn build_widget_frame(snapshot: &AppStatusSnapshot, config: &AppConfig, rect: &RECT) -> WidgetFrame {
    let width = (rect.right - rect.left).max(0);
    let height = (rect.bottom - rect.top).max(0);
    let mut buffer = PixelBuffer::new(width, height);
    let palette = Palette::from_config(&config.widget_visual.palette);
    let groups = build_group_layouts(snapshot, config, width, height);

    for group in &groups {
        draw_group(&mut buffer, group, palette);
    }

    WidgetFrame {
        hot_zones: groups.into_iter().map(|group| WidgetHotZone {
            group: group.id,
            rect: group.hot_zone,
        }).collect(),
        width,
        height,
        pixels: buffer.pixels,
    }
}

pub fn apply_widget_frame(hwnd: HWND, frame: &WidgetFrame) {
    if frame.width <= 0 || frame.height <= 0 {
        return;
    }

    unsafe {
        let screen_dc = GetDC(HWND::default());
        if screen_dc.0.is_null() {
            return;
        }
        let mem_dc = CreateCompatibleDC(screen_dc);
        if mem_dc.0.is_null() {
            let _ = ReleaseDC(HWND::default(), screen_dc);
            return;
        }

        let bitmap_info = BITMAPINFO {
            bmiHeader: BITMAPINFOHEADER {
                biSize: std::mem::size_of::<BITMAPINFOHEADER>() as u32,
                biWidth: frame.width,
                biHeight: -frame.height,
                biPlanes: 1,
                biBitCount: 32,
                biCompression: BI_RGB.0,
                ..Default::default()
            },
            bmiColors: [RGBQUAD::default(); 1],
        };
        let mut bits = std::ptr::null_mut();
        let Ok(bitmap) = CreateDIBSection(
            mem_dc,
            &bitmap_info,
            DIB_RGB_COLORS,
            &mut bits,
            None,
            0,
        ) else {
            let _ = DeleteDC(mem_dc);
            let _ = ReleaseDC(HWND::default(), screen_dc);
            return;
        };
        if bits.is_null() {
            let _ = DeleteObject(HGDIOBJ(bitmap.0));
            let _ = DeleteDC(mem_dc);
            let _ = ReleaseDC(HWND::default(), screen_dc);
            return;
        }

        std::ptr::copy_nonoverlapping(frame.pixels.as_ptr(), bits.cast::<u8>(), frame.pixels.len());
        let previous = SelectObject(mem_dc, HGDIOBJ(bitmap.0));
        let source_point = POINT { x: 0, y: 0 };
        let size = SIZE {
            cx: frame.width,
            cy: frame.height,
        };
        let blend = BLENDFUNCTION {
            BlendOp: AC_SRC_OVER as u8,
            BlendFlags: 0,
            SourceConstantAlpha: 255,
            AlphaFormat: AC_SRC_ALPHA as u8,
        };

        let _ = UpdateLayeredWindow(
            hwnd,
            screen_dc,
            None,
            Some(&size),
            mem_dc,
            Some(&source_point),
            COLORREF(0),
            Some(&blend),
            ULW_ALPHA,
        );

        let _ = SelectObject(mem_dc, previous);
        let _ = DeleteObject(HGDIOBJ(bitmap.0));
        let _ = DeleteDC(mem_dc);
        let _ = ReleaseDC(HWND::default(), screen_dc);
    }
}

pub fn hit_test(hot_zones: &[WidgetHotZone], point: POINT) -> Option<WidgetGroupId> {
    hot_zones.iter().find_map(|zone| point_in_rect(point, zone.rect).then_some(zone.group))
}

fn build_group_layouts(
    snapshot: &AppStatusSnapshot,
    config: &AppConfig,
    width: i32,
    height: i32,
) -> Vec<GroupLayout> {
    let mut visible_groups = Vec::new();

    if config.monitoring.codex_enabled {
        visible_groups.push((WidgetGroupId::Codex, 'C', source_state(snapshot, "codex")));
    }
    if config.monitoring.claude_enabled {
        visible_groups.push((WidgetGroupId::Claude, 'L', source_state(snapshot, "claude")));
    }

    let cell_count = visible_groups.len().max(1) as i32;
    let cell_width = if cell_count == 0 { width } else { width / cell_count.max(1) };
    let group_width = 56;
    let group_height = 34;
    let lamp_radius = 5.5_f32;
    let label_width = 14;
    let lamp_spacing = 14;
    let label_gap = 8;
    let group_content_width = label_width + label_gap + lamp_spacing * 2 + 12;

    visible_groups
        .into_iter()
        .enumerate()
        .map(|(index, (id, label, state))| {
            let cell_left = index as i32 * cell_width;
            let cell_right = if index + 1 == cell_count as usize {
                width
            } else {
                (index as i32 + 1) * cell_width
            };
            let cell_center_x = (cell_left + cell_right) / 2;
            let content_left = cell_center_x - (group_content_width / 2);
            let group_top = ((height - group_height) / 2).max(0);
            let hot_zone = RECT {
                left: (cell_left + 4).max(0),
                top: (group_top - 2).max(0),
                right: (cell_right - 4).max((cell_left + group_width).min(width)),
                bottom: (group_top + group_height + 2).min(height),
            };
            let lamp_center_y = group_top as f32 + 20.0;
            let first_center_x = content_left as f32 + label_width as f32 + label_gap as f32 + lamp_radius;

            GroupLayout {
                id,
                label,
                state,
                hot_zone,
                lights: [
                    LampLayout {
                        center_x: first_center_x,
                        center_y: lamp_center_y,
                        radius: lamp_radius,
                    },
                    LampLayout {
                        center_x: first_center_x + lamp_spacing as f32,
                        center_y: lamp_center_y,
                        radius: lamp_radius,
                    },
                    LampLayout {
                        center_x: first_center_x + (lamp_spacing * 2) as f32,
                        center_y: lamp_center_y,
                        radius: lamp_radius,
                    },
                ],
            }
        })
        .collect()
}

fn draw_group(buffer: &mut PixelBuffer, group: &GroupLayout, palette: Palette) {
    draw_label(buffer, group.label, group.hot_zone.left + 8, group.hot_zone.top + 10, scale_color(palette.off, 0.85), 2);

    for (index, lamp) in group.lights.iter().enumerate() {
        let is_active = lamp_is_active(group.state, index);
        let base_color = match index {
            0 => {
                if is_active { palette.green } else { palette.off }
            }
            1 => {
                if is_active { palette.yellow } else { palette.off }
            }
            _ => {
                if is_active { palette.red } else { palette.off }
            }
        };
        let glow = if is_active {
            with_alpha(base_color, 72)
        } else {
            with_alpha(base_color, 36)
        };
        let specular = if is_active {
            Rgba {
                red: 255,
                green: 255,
                blue: 255,
                alpha: 54,
            }
        } else {
            Rgba {
                red: 255,
                green: 255,
                blue: 255,
                alpha: 22,
            }
        };

        draw_circle(buffer, lamp.center_x, lamp.center_y + 0.5, lamp.radius + 2.2, glow);
        draw_circle(buffer, lamp.center_x, lamp.center_y, lamp.radius, with_alpha(base_color, 245));
        draw_circle(buffer, lamp.center_x - 1.35, lamp.center_y - 1.5, lamp.radius * 0.48, specular);
    }
}

fn lamp_is_active(state: SourceVisualState, lamp_index: usize) -> bool {
    match state {
        SourceVisualState::Idle | SourceVisualState::Working => lamp_index == 0,
        SourceVisualState::Completed | SourceVisualState::NeedsAttention => lamp_index == 1,
        SourceVisualState::Error => lamp_index == 2,
    }
}

fn source_state(snapshot: &AppStatusSnapshot, key: &str) -> SourceVisualState {
    snapshot
        .sources
        .get(key)
        .map(|source| source.state)
        .unwrap_or(SourceVisualState::Idle)
}

fn point_in_rect(point: POINT, rect: RECT) -> bool {
    point.x >= rect.left && point.x < rect.right && point.y >= rect.top && point.y < rect.bottom
}

fn draw_label(buffer: &mut PixelBuffer, label: char, left: i32, top: i32, color: Rgba, scale: i32) {
    let glyph = match label {
        'C' => ["01110", "10001", "10000", "10001", "01110"],
        'L' => ["10000", "10000", "10000", "10000", "11110"],
        _ => ["00000", "00000", "00000", "00000", "00000"],
    };

    for (row_index, row) in glyph.iter().enumerate() {
        for (column_index, cell) in row.chars().enumerate() {
            if cell == '1' {
                for dy in 0..scale {
                    for dx in 0..scale {
                        buffer.blend_pixel(
                            left + column_index as i32 * scale + dx,
                            top + row_index as i32 * scale + dy,
                            color,
                        );
                    }
                }
            }
        }
    }
}

fn draw_circle(buffer: &mut PixelBuffer, center_x: f32, center_y: f32, radius: f32, color: Rgba) {
    let min_x = (center_x - radius - 1.0).floor() as i32;
    let max_x = (center_x + radius + 1.0).ceil() as i32;
    let min_y = (center_y - radius - 1.0).floor() as i32;
    let max_y = (center_y + radius + 1.0).ceil() as i32;
    let radius_squared = radius * radius;

    for y in min_y..=max_y {
        for x in min_x..=max_x {
            let mut covered_samples = 0;
            for (sample_x, sample_y) in SUPERSAMPLE_GRID {
                let dx = x as f32 + sample_x - center_x;
                let dy = y as f32 + sample_y - center_y;
                if dx * dx + dy * dy <= radius_squared {
                    covered_samples += 1;
                }
            }

            if covered_samples == 0 {
                continue;
            }

            let alpha = ((u32::from(color.alpha) * covered_samples) / SUPERSAMPLE_GRID.len() as u32) as u8;
            buffer.blend_pixel(x, y, with_alpha(color, alpha));
        }
    }
}

impl PixelBuffer {
    fn new(width: i32, height: i32) -> Self {
        let len = width.max(0) as usize * height.max(0) as usize * 4;
        Self {
            width,
            height,
            pixels: vec![0; len],
        }
    }

    fn blend_pixel(&mut self, x: i32, y: i32, color: Rgba) {
        if x < 0 || y < 0 || x >= self.width || y >= self.height || color.alpha == 0 {
            return;
        }

        let index = ((y as usize * self.width as usize) + x as usize) * 4;
        let dst_blue = self.pixels[index] as u32;
        let dst_green = self.pixels[index + 1] as u32;
        let dst_red = self.pixels[index + 2] as u32;
        let dst_alpha = self.pixels[index + 3] as u32;

        let src_alpha = u32::from(color.alpha);
        let inv_alpha = 255 - src_alpha;
        let src_blue = (u32::from(color.blue) * src_alpha) / 255;
        let src_green = (u32::from(color.green) * src_alpha) / 255;
        let src_red = (u32::from(color.red) * src_alpha) / 255;

        self.pixels[index] = (src_blue + (dst_blue * inv_alpha) / 255) as u8;
        self.pixels[index + 1] = (src_green + (dst_green * inv_alpha) / 255) as u8;
        self.pixels[index + 2] = (src_red + (dst_red * inv_alpha) / 255) as u8;
        self.pixels[index + 3] = (src_alpha + (dst_alpha * inv_alpha) / 255) as u8;
    }
}

impl Palette {
    fn from_config(config: &WidgetPaletteConfig) -> Self {
        Self {
            green: parse_hex_color(&config.green).unwrap_or(Rgba {
                red: 82,
                green: 214,
                blue: 113,
                alpha: 255,
            }),
            yellow: parse_hex_color(&config.yellow).unwrap_or(Rgba {
                red: 255,
                green: 210,
                blue: 76,
                alpha: 255,
            }),
            red: parse_hex_color(&config.red).unwrap_or(Rgba {
                red: 255,
                green: 108,
                blue: 96,
                alpha: 255,
            }),
            off: parse_hex_color(&config.off).unwrap_or(Rgba {
                red: 48,
                green: 48,
                blue: 52,
                alpha: 255,
            }),
        }
    }
}

fn parse_hex_color(value: &str) -> Option<Rgba> {
    let hex = value.trim().trim_start_matches('#');
    if hex.len() != 6 {
        return None;
    }

    let red = u8::from_str_radix(&hex[0..2], 16).ok()?;
    let green = u8::from_str_radix(&hex[2..4], 16).ok()?;
    let blue = u8::from_str_radix(&hex[4..6], 16).ok()?;
    Some(Rgba {
        red,
        green,
        blue,
        alpha: 255,
    })
}

fn with_alpha(color: Rgba, alpha: u8) -> Rgba {
    Rgba {
        alpha,
        ..color
    }
}

fn scale_color(color: Rgba, factor: f32) -> Rgba {
    Rgba {
        red: (color.red as f32 * factor).round().clamp(0.0, 255.0) as u8,
        green: (color.green as f32 * factor).round().clamp(0.0, 255.0) as u8,
        blue: (color.blue as f32 * factor).round().clamp(0.0, 255.0) as u8,
        alpha: color.alpha,
    }
}
