use crate::{
    app_config::{AppConfig, WidgetPaletteConfig},
    ui_state::AppStatusSnapshot,
    widget_effects::{GroupRenderState, WidgetEffectsState},
    widget_image,
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
pub(crate) struct Rgba {
    pub(crate) red: u8,
    pub(crate) green: u8,
    pub(crate) blue: u8,
    pub(crate) alpha: u8,
}

pub(crate) struct PixelBuffer {
    width: i32,
    height: i32,
    pixels: Vec<u8>,
}

struct GroupLayout {
    id: WidgetGroupId,
    logo_left: i32,
    logo_top: i32,
    render_state: GroupRenderState,
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
    inactive_brightness_percent: u8,
}

#[derive(Clone, Copy)]
struct LampPaintRecipe {
    active_base: Rgba,
    inactive_brightness: f32,
    inactive_border_alpha: u8,
    active_glow_alpha: u8,
}

/// Expected logo width when show_labels=true (normal case).
/// Used at compile time to size the widget; actual decoded dimensions
/// are queried at runtime during layout.
const LOGO_EXPECTED_WIDTH: i32 = 16;

/// Total width of the three-lamp track: left half-gap + 3 lamps + right half-gap.
/// Computed from LAMP_RADIUS and LAMP_SPACING to avoid drift.
const LAMP_TRACK_WIDTH: i32 = (LAMP_RADIUS * 2.0).ceil() as i32 + LAMP_SPACING * 2;

/// Full width of one group cell when logos are shown.
/// Automatically adapts when LAMP_RADIUS, LAMP_SPACING, or LOGO_GAP change.
/// NOTE: If you replace the logo PNGs, this const should match the actual
/// logo width.  The rendering code reads the decoded dimensions dynamically,
/// but the widget's window size uses this compile-time value.
/// For fully automatic sizing, use group_width() instead.
pub const GROUP_WIDTH: i32 = LOGO_EXPECTED_WIDTH + LOGO_GAP + LAMP_TRACK_WIDTH;

/// Compute the actual group width from the decoded logo dimensions at runtime.
/// This is the content width per group (not including margins).
pub fn group_width() -> i32 {
    let logo_w = widget_image::get_logo(WidgetGroupId::Codex).width as i32;
    logo_w + LOGO_GAP + LAMP_TRACK_WIDTH
}

/// Total widget width for a given number of groups, including margins between them.
pub fn total_widget_width(group_count: usize) -> i32 {
    let per_group = group_width();
    let count = group_count.max(1) as i32;
    if count <= 1 {
        per_group
    } else {
        per_group * count + (count - 1) * GROUP_MARGIN
    }
}
const GROUP_HEIGHT: i32 = 40;
const LAMP_RADIUS: f32 = 8.0;
const LAMP_SPACING: i32 = 24;
const LOGO_GAP: i32 = 8; // logo 到第一个灯的间距

/// Horizontal margin (px) between adjacent group cells.
pub const GROUP_MARGIN: i32 = 20; // 灯组之间的水平外边距（px）

/// Width of the vertical divider line, in pixels.
const DIVIDER_WIDTH: i32 = 3; // 分隔线的宽度（px）

/// Divider height as a percentage of the widget height (0-100).
/// At 100 it spans the full widget height; at smaller values it is centered.
const DIVIDER_HEIGHT_PERCENT: u8 = 50; // 分隔线高度占 widget 高度的百分比（0-100）

/// Divider colour: white semi-transparent.
const DIVIDER_COLOR: Rgba = Rgba {
    red: 255,
    green: 255,
    blue: 255,
    alpha: 30,
};

const LEGACY_GREEN: Rgba = rgba(82, 214, 113, 255);
const LEGACY_YELLOW: Rgba = rgba(255, 210, 76, 255);
const LEGACY_RED: Rgba = rgba(255, 108, 96, 255);
// Keep renderer defaults aligned with shared-core app_config defaults and the
// Tauri settings reset palette so one active color system stays authoritative.
const DEFAULT_GREEN: Rgba = rgba(52, 199, 89, 255); // #34C759
const DEFAULT_YELLOW: Rgba = rgba(255, 204, 0, 255); // #FFCC00
const DEFAULT_RED: Rgba = rgba(255, 59, 48, 255); // #FF3B30
const AMBIENT_OFF: Rgba = rgba(48, 48, 52, 255);

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

pub fn build_widget_frame(
    snapshot: &AppStatusSnapshot,
    effects: &WidgetEffectsState,
    now_ms: u64,
    config: &AppConfig,
    rect: &RECT,
) -> WidgetFrame {
    let width = (rect.right - rect.left).max(0);
    let height = (rect.bottom - rect.top).max(0);
    let mut buffer = PixelBuffer::new(width, height);
    let palette = Palette::from_config(&config.widget_visual.palette);
    let groups = build_group_layouts(snapshot, effects, now_ms, config, width, height);

    for group in &groups {
        draw_group(&mut buffer, group, palette);
    }

    // Vertical divider between groups.
    // Drawn in the gap between cells, with configurable width and height.
    if groups.len() > 1 {
        let cell_content_width = width - (groups.len() as i32 - 1) * GROUP_MARGIN;
        let cell_count = groups.len() as i32;
        let cell_width = cell_content_width / cell_count;
        let divider_h = (height * DIVIDER_HEIGHT_PERCENT as i32) / 100;
        let divider_top = (height - divider_h) / 2;
        let divider_bottom = divider_top + divider_h;

        for i in 1..cell_count {
            // Divider sits in the centre of the margin gap.
            let gap_center = i * (cell_width + GROUP_MARGIN) - GROUP_MARGIN / 2;
            let divider_left = gap_center - DIVIDER_WIDTH / 2;
            for dx in 0..DIVIDER_WIDTH {
                draw_divider(&mut buffer, divider_left + dx, divider_top, divider_bottom);
            }
        }
    }

    WidgetFrame {
        hot_zones: groups
            .into_iter()
            .map(|group| WidgetHotZone {
                group: group.id,
                rect: group.hot_zone,
            })
            .collect(),
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
        let Ok(bitmap) = CreateDIBSection(mem_dc, &bitmap_info, DIB_RGB_COLORS, &mut bits, None, 0)
        else {
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
    hot_zones
        .iter()
        .find_map(|zone| point_in_rect(point, zone.rect).then_some(zone.group))
}

fn build_group_layouts(
    snapshot: &AppStatusSnapshot,
    effects: &WidgetEffectsState,
    now_ms: u64,
    config: &AppConfig,
    width: i32,
    height: i32,
) -> Vec<GroupLayout> {
    let mut visible_groups = Vec::new();

    if config.monitoring.codex_enabled {
        visible_groups.push((
            WidgetGroupId::Codex,
            effects.render_state_for(snapshot, "codex", now_ms),
        ));
    }
    if config.monitoring.claude_enabled {
        visible_groups.push((
            WidgetGroupId::Claude,
            effects.render_state_for(snapshot, "claude", now_ms),
        ));
    }

    let cell_count = visible_groups.len().max(1) as i32;
    // Total content area width excludes margins (those are accounted for in
    // the total widget width computed by total_widget_width()).
    let cell_content_width = if cell_count == 0 {
        width
    } else {
        let margin_total = (cell_count - 1).max(0) * GROUP_MARGIN;
        (width - margin_total).max(0) / cell_count
    };
    let logo_width = {
        let first_id = visible_groups
            .first()
            .map(|(id, _)| *id)
            .unwrap_or(WidgetGroupId::Codex);
        widget_image::get_logo(first_id).width as i32
    };
    let logo_gap = LOGO_GAP;
    let group_content_width = logo_width + logo_gap + LAMP_TRACK_WIDTH;

    visible_groups
        .into_iter()
        .enumerate()
        .map(|(index, (id, render_state))| {
            let cell_left = index as i32 * (cell_content_width + GROUP_MARGIN);
            let cell_right = cell_left + cell_content_width;
            let cell_center_x = (cell_left + cell_right) / 2;
            let content_left = cell_center_x - (group_content_width / 2);
            let group_top = ((height - GROUP_HEIGHT) / 2).max(0);
            let logo_h = widget_image::get_logo(id).height as i32;
            let hot_zone = RECT {
                left: (cell_left + 4).max(0),
                top: (group_top - 2).max(0),
                right: (cell_right - 4).max((cell_left + GROUP_WIDTH).min(width)),
                bottom: (group_top + GROUP_HEIGHT + 2).min(height),
            };
            let lamp_center_y = group_top as f32 + 19.5;
            let first_center_x =
                content_left as f32 + logo_width as f32 + logo_gap as f32 + LAMP_RADIUS;

            GroupLayout {
                id,
                logo_left: content_left,
                logo_top: group_top + (GROUP_HEIGHT - logo_h) / 2,
                render_state,
                hot_zone,
                lights: [
                    LampLayout {
                        center_x: first_center_x,
                        center_y: lamp_center_y,
                        radius: LAMP_RADIUS,
                    },
                    LampLayout {
                        center_x: first_center_x + LAMP_SPACING as f32,
                        center_y: lamp_center_y,
                        radius: LAMP_RADIUS,
                    },
                    LampLayout {
                        center_x: first_center_x + (LAMP_SPACING * 2) as f32,
                        center_y: lamp_center_y,
                        radius: LAMP_RADIUS,
                    },
                ],
            }
        })
        .collect()
}

fn draw_group(buffer: &mut PixelBuffer, group: &GroupLayout, palette: Palette) {
    let logo = widget_image::get_logo(group.id);
    widget_image::blit_logo(buffer, group.logo_left, group.logo_top, logo);

    for (index, lamp) in group.lights.iter().enumerate() {
        let lamp_state = group.render_state.lamps[index];
        let recipe = lamp_paint_recipe(index, palette);
        draw_idle_lamp(buffer, lamp, recipe);

        if lamp_state.alpha > 0 {
            draw_active_lamp(buffer, lamp, recipe, lamp_state.alpha);
        }
    }
}

// Keep every indicator as a clean circular shell. Only active states receive bloom.
fn draw_idle_lamp(buffer: &mut PixelBuffer, lamp: &LampLayout, recipe: LampPaintRecipe) {
    let inactive_base = scale_color(recipe.active_base, recipe.inactive_brightness);
    let border = with_alpha(recipe.active_base, recipe.inactive_border_alpha);
    let body = with_alpha(inactive_base, 212);
    let inner_recess = with_alpha(blend_colors(inactive_base, AMBIENT_OFF, 0.24), 184);
    let center_shadow = with_alpha(blend_colors(inactive_base, AMBIENT_OFF, 0.42), 150);
    let specular = Rgba {
        red: 255,
        green: 255,
        blue: 255,
        alpha: 12,
    };

    draw_circle(
        buffer,
        lamp.center_x,
        lamp.center_y,
        lamp.radius + 0.2,
        border,
    );
    draw_circle(
        buffer,
        lamp.center_x,
        lamp.center_y,
        lamp.radius - 0.55,
        body,
    );
    draw_circle(
        buffer,
        lamp.center_x,
        lamp.center_y,
        lamp.radius * 0.66,
        inner_recess,
    );
    draw_circle(
        buffer,
        lamp.center_x,
        lamp.center_y,
        lamp.radius * 0.34,
        center_shadow,
    );
    draw_circle(
        buffer,
        lamp.center_x - 1.35,
        lamp.center_y - 1.45,
        lamp.radius * 0.22,
        specular,
    );
}

fn draw_active_lamp(
    buffer: &mut PixelBuffer,
    lamp: &LampLayout,
    recipe: LampPaintRecipe,
    alpha: u8,
) {
    let active_base = recipe.active_base;
    let outer_halo = with_alpha(
        active_base,
        scale_alpha(alpha, recipe.active_glow_alpha / 4),
    );
    let mid_halo = with_alpha(
        active_base,
        scale_alpha(alpha, recipe.active_glow_alpha / 2),
    );
    let near_halo = with_alpha(active_base, scale_alpha(alpha, recipe.active_glow_alpha));
    let body = with_alpha(scale_color(active_base, 1.02), scale_alpha(alpha, 228));
    let inner_core = with_alpha(scale_color(active_base, 1.16), scale_alpha(alpha, 255));
    let specular = Rgba {
        red: 255,
        green: 255,
        blue: 255,
        alpha: scale_alpha(alpha, 34),
    };

    // Three concentric bloom layers simulate a soft Gaussian hardware LED falloff.
    draw_circle(
        buffer,
        lamp.center_x,
        lamp.center_y,
        lamp.radius + 8.5,
        outer_halo,
    );
    draw_circle(
        buffer,
        lamp.center_x,
        lamp.center_y,
        lamp.radius + 5.5,
        mid_halo,
    );
    draw_circle(
        buffer,
        lamp.center_x,
        lamp.center_y,
        lamp.radius + 2.5,
        near_halo,
    );
    draw_circle(buffer, lamp.center_x, lamp.center_y, lamp.radius, body);
    draw_circle(
        buffer,
        lamp.center_x,
        lamp.center_y,
        lamp.radius * 0.52,
        inner_core,
    );
    draw_circle(
        buffer,
        lamp.center_x - 1.35,
        lamp.center_y - 1.45,
        lamp.radius * 0.2,
        specular,
    );
}

fn point_in_rect(point: POINT, rect: RECT) -> bool {
    point.x >= rect.left && point.x < rect.right && point.y >= rect.top && point.y < rect.bottom
}

fn draw_divider(buffer: &mut PixelBuffer, x: i32, y0: i32, y1: i32) {
    if x < 0 || x >= buffer.width() {
        return;
    }
    let y_start = y0.max(0);
    let y_end = y1.min(buffer.height());
    for y in y_start..y_end {
        buffer.blend_pixel(x, y, DIVIDER_COLOR);
    }
}

#[allow(dead_code)]
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

            let alpha =
                ((u32::from(color.alpha) * covered_samples) / SUPERSAMPLE_GRID.len() as u32) as u8;
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

    pub(crate) fn width(&self) -> i32 {
        self.width
    }

    pub(crate) fn height(&self) -> i32 {
        self.height
    }

    pub(crate) fn blend_pixel(&mut self, x: i32, y: i32, color: Rgba) {
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
            green: parse_hex_color(&config.green).unwrap_or(DEFAULT_GREEN),
            yellow: parse_hex_color(&config.yellow).unwrap_or(DEFAULT_YELLOW),
            red: parse_hex_color(&config.red).unwrap_or(DEFAULT_RED),
            inactive_brightness_percent: config.inactive_brightness_percent.clamp(12, 80),
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

const fn rgba(red: u8, green: u8, blue: u8, alpha: u8) -> Rgba {
    Rgba {
        red,
        green,
        blue,
        alpha,
    }
}

fn lamp_paint_recipe(index: usize, palette: Palette) -> LampPaintRecipe {
    let inactive_brightness = palette.inactive_brightness_percent as f32 / 100.0;
    match index {
        0 => LampPaintRecipe {
            active_base: normalize_legacy_active_color(palette.green, LEGACY_GREEN, DEFAULT_GREEN),
            inactive_brightness,
            inactive_border_alpha: 26,
            active_glow_alpha: 115,
        },
        1 => LampPaintRecipe {
            active_base: normalize_legacy_active_color(
                palette.yellow,
                LEGACY_YELLOW,
                DEFAULT_YELLOW,
            ),
            inactive_brightness,
            inactive_border_alpha: 25,
            active_glow_alpha: 102,
        },
        _ => LampPaintRecipe {
            active_base: normalize_legacy_active_color(palette.red, LEGACY_RED, DEFAULT_RED),
            inactive_brightness,
            inactive_border_alpha: 31,
            active_glow_alpha: 115,
        },
    }
}

fn normalize_legacy_active_color(current: Rgba, legacy_default: Rgba, new_default: Rgba) -> Rgba {
    if same_rgb(current, legacy_default) {
        new_default
    } else {
        current
    }
}

fn same_rgb(left: Rgba, right: Rgba) -> bool {
    left.red == right.red && left.green == right.green && left.blue == right.blue
}

fn with_alpha(color: Rgba, alpha: u8) -> Rgba {
    Rgba { alpha, ..color }
}

fn scale_alpha(alpha: u8, target_alpha: u8) -> u8 {
    ((u16::from(alpha) * u16::from(target_alpha)) / 255) as u8
}

fn blend_colors(base: Rgba, overlay: Rgba, overlay_weight: f32) -> Rgba {
    let weight = overlay_weight.clamp(0.0, 1.0);
    let inv = 1.0 - weight;
    Rgba {
        red: (base.red as f32 * inv + overlay.red as f32 * weight).round() as u8,
        green: (base.green as f32 * inv + overlay.green as f32 * weight).round() as u8,
        blue: (base.blue as f32 * inv + overlay.blue as f32 * weight).round() as u8,
        alpha: base.alpha,
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        app_config::AppConfig,
        ui_state::{AppStatusSnapshot, SourceVisualState},
        widget_effects::WidgetEffectsState,
    };

    fn snapshot_with_state(state: SourceVisualState) -> AppStatusSnapshot {
        let mut snapshot = AppStatusSnapshot::empty();
        snapshot
            .sources
            .get_mut("codex")
            .expect("codex source")
            .state = state;
        snapshot.overall_state = state;
        snapshot
    }

    fn render_frame(
        width: i32,
        height: i32,
        state: SourceVisualState,
        configure: impl FnOnce(&mut AppConfig),
    ) -> WidgetFrame {
        let mut config = AppConfig::default_v1();
        configure(&mut config);
        let rect = RECT {
            left: 0,
            top: 0,
            right: width,
            bottom: height,
        };
        let snapshot = snapshot_with_state(state);

        build_widget_frame(&snapshot, &WidgetEffectsState::default(), 0, &config, &rect)
    }

    fn has_visible_pixel(
        frame: &WidgetFrame,
        left: i32,
        top: i32,
        right: i32,
        bottom: i32,
    ) -> bool {
        for y in top.max(0)..bottom.min(frame.height) {
            for x in left.max(0)..right.min(frame.width) {
                let index = ((y as usize * frame.width as usize) + x as usize) * 4 + 3;
                if frame.pixels[index] > 0 {
                    return true;
                }
            }
        }

        false
    }

    fn pixel_rgba(frame: &WidgetFrame, x: i32, y: i32) -> (u8, u8, u8, u8) {
        let index = ((y as usize * frame.width as usize) + x as usize) * 4;
        (
            frame.pixels[index + 2],
            frame.pixels[index + 1],
            frame.pixels[index],
            frame.pixels[index + 3],
        )
    }

    fn single_group_layout(state: SourceVisualState) -> GroupLayout {
        let mut config = AppConfig::default_v1();
        config.monitoring.claude_enabled = false;

        build_group_layouts(
            &snapshot_with_state(state),
            &WidgetEffectsState::default(),
            0,
            &config,
            GROUP_WIDTH,
            32,
        )
        .into_iter()
        .next()
        .expect("single group layout")
    }

    fn lamp_bounds(layout: &GroupLayout, index: usize) -> (i32, i32, i32, i32) {
        let lamp = layout.lights[index];
        let radius = (lamp.radius + 6.0).ceil() as i32;
        let center_x = lamp.center_x.round() as i32;
        let center_y = lamp.center_y.round() as i32;

        (
            center_x - radius,
            center_y - radius,
            center_x + radius,
            center_y + radius,
        )
    }

    fn lamp_center(layout: &GroupLayout, index: usize) -> (i32, i32) {
        let lamp = layout.lights[index];
        (lamp.center_x.round() as i32, lamp.center_y.round() as i32)
    }

    #[test]
    fn idle_single_group_still_draws_off_lamps() {
        let layout = single_group_layout(SourceVisualState::Idle);
        let frame = render_frame(GROUP_WIDTH, 32, SourceVisualState::Idle, |config| {
            config.monitoring.claude_enabled = false;
        });

        let first = lamp_bounds(&layout, 0);
        let second = lamp_bounds(&layout, 1);
        let third = lamp_bounds(&layout, 2);

        assert!(
            has_visible_pixel(&frame, first.0, first.1, first.2, first.3),
            "idle frame should still render the first off lamp"
        );
        assert!(
            has_visible_pixel(&frame, second.0, second.1, second.2, second.3),
            "idle frame should still render the second off lamp"
        );
        assert!(
            has_visible_pixel(&frame, third.0, third.1, third.2, third.3),
            "idle frame should still render the third off lamp"
        );
    }

    #[test]
    fn working_single_group_keeps_all_three_lamp_positions_visible() {
        let layout = single_group_layout(SourceVisualState::Working);
        assert_eq!(layout.render_state.lamps[0].alpha, 255);
        assert_eq!(layout.render_state.lamps[1].alpha, 0);
        assert_eq!(layout.render_state.lamps[2].alpha, 0);

        let frame = render_frame(GROUP_WIDTH, 32, SourceVisualState::Working, |config| {
            config.monitoring.claude_enabled = false;
        });

        let first = lamp_bounds(&layout, 0);
        let second = lamp_bounds(&layout, 1);
        let third = lamp_bounds(&layout, 2);
        assert!(has_visible_pixel(
            &frame, first.0, first.1, first.2, first.3
        ));
        assert!(has_visible_pixel(
            &frame, second.0, second.1, second.2, second.3
        ));
        assert!(has_visible_pixel(
            &frame, third.0, third.1, third.2, third.3
        ));

        let left_center_coords = lamp_center(&layout, 0);
        let mid_center_coords = lamp_center(&layout, 1);
        let left_center = pixel_rgba(&frame, left_center_coords.0, left_center_coords.1);
        let mid_center = pixel_rgba(&frame, mid_center_coords.0, mid_center_coords.1);
        let (left_red, left_green, left_blue, left_alpha) =
            (left_center.0, left_center.1, left_center.2, left_center.3);
        let (mid_red, mid_green, mid_blue, _mid_alpha) =
            (mid_center.0, mid_center.1, mid_center.2, mid_center.3);

        assert!(left_alpha > 0, "working left lamp should render");
        assert!(
            left_green > left_red && left_green > left_blue,
            "working state should color the left lamp green (left_center={left_center:?})"
        );
        assert!(
            u16::from(mid_red) + u16::from(mid_green) + u16::from(mid_blue)
                < u16::from(left_red) + u16::from(left_green) + u16::from(left_blue),
            "working state should keep inactive lamps visibly dimmer than the active lamp (left_center={left_center:?}, mid_center={mid_center:?})"
        );
    }

    #[test]
    fn completed_single_group_keeps_all_three_lamp_positions_visible() {
        let layout = single_group_layout(SourceVisualState::Completed);
        assert_eq!(layout.render_state.lamps[0].alpha, 255);
        assert_eq!(layout.render_state.lamps[1].alpha, 0);
        assert_eq!(layout.render_state.lamps[2].alpha, 0);

        let frame = render_frame(GROUP_WIDTH, 32, SourceVisualState::Completed, |config| {
            config.monitoring.claude_enabled = false;
        });

        let first = lamp_bounds(&layout, 0);
        let second = lamp_bounds(&layout, 1);
        let third = lamp_bounds(&layout, 2);
        assert!(has_visible_pixel(
            &frame, first.0, first.1, first.2, first.3
        ));
        assert!(has_visible_pixel(
            &frame, second.0, second.1, second.2, second.3
        ));
        assert!(has_visible_pixel(
            &frame, third.0, third.1, third.2, third.3
        ));

        let left_center_coords = lamp_center(&layout, 0);
        let mid_center_coords = lamp_center(&layout, 1);
        let left_center = pixel_rgba(&frame, left_center_coords.0, left_center_coords.1);
        let mid_center = pixel_rgba(&frame, mid_center_coords.0, mid_center_coords.1);
        let (left_red, left_green, left_blue, _left_alpha) =
            (left_center.0, left_center.1, left_center.2, left_center.3);
        let (mid_red, mid_green, mid_blue, _mid_alpha) =
            (mid_center.0, mid_center.1, mid_center.2, mid_center.3);

        assert!(
            left_green > left_red && left_green > left_blue,
            "completed state should keep the left lamp steadily green per README (left_center={left_center:?})"
        );
        assert!(
            u16::from(mid_red) + u16::from(mid_green) + u16::from(mid_blue)
                < u16::from(left_red) + u16::from(left_green) + u16::from(left_blue),
            "completed state should keep inactive lamps visibly dimmer than the active lamp (left_center={left_center:?}, mid_center={mid_center:?})"
        );
    }

    #[test]
    fn logo_is_rendered() {
        let frame = render_frame(GROUP_WIDTH, 48, SourceVisualState::Idle, |config| {
            config.monitoring.claude_enabled = false;
        });
        assert!(
            has_visible_pixel(&frame, 0, 12, 22, 36),
            "logo should always render"
        );
    }
}
