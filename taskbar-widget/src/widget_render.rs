use crate::{
    app_config::{AppConfig, WidgetPaletteConfig},
    ui_state::AppStatusSnapshot,
    widget_effects::{GroupRenderState, WidgetEffectsState},
    widget_image,
};
use std::sync::Arc;
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

impl WidgetGroupId {
    fn source_id(self) -> &'static str {
        match self {
            Self::Codex => "codex",
            Self::Claude => "claude",
        }
    }
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
    cell_right: i32,
    logo_left: i32,
    logo_top: i32,
    render_state: GroupRenderState,
    hot_zone: RECT,
    lights: [LampLayout; 3],
    material: Option<Arc<widget_image::MaterialGroupImages>>,
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
struct MaterialBrightness {
    idle_percent: u8,
    blink_percent: u8,
    steady_percent: u8,
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

/// Compute the actual built-in lamp group width from the decoded logo dimensions.
pub fn group_width() -> i32 {
    let logo_w = widget_image::get_logo(WidgetGroupId::Codex).width as i32;
    logo_w + LOGO_GAP + LAMP_TRACK_WIDTH
}

const GROUP_HEIGHT: i32 = 40;
const LAMP_RADIUS: f32 = 8.0;
const LAMP_SPACING: i32 = 24;
const MATERIAL_GAP: i32 = 8;
const MATERIAL_VERTICAL_PADDING: i32 = 8;
const LOGO_GAP: i32 = 8; // logo 到第一个灯的间距

/// Horizontal margin (px) between adjacent group cells.
pub const GROUP_MARGIN: i32 = 20; // 灯组之间的水平外边距（px）

fn visible_group_ids(config: &AppConfig) -> Vec<WidgetGroupId> {
    let mut ids = Vec::new();
    if config.monitoring.codex_enabled {
        ids.push(WidgetGroupId::Codex);
    }
    if config.monitoring.claude_enabled {
        ids.push(WidgetGroupId::Claude);
    }
    ids
}

fn selected_material_group<'a>(
    config: &'a AppConfig,
    id: WidgetGroupId,
) -> Option<&'a crate::app_config::MaterialGroup> {
    let selected_id = match id {
        WidgetGroupId::Codex => config.widget_visual.codex_material_group_id.as_deref(),
        WidgetGroupId::Claude => config.widget_visual.claude_material_group_id.as_deref(),
    }?;
    config
        .widget_visual
        .material_groups
        .iter()
        .find(|group| group.id == selected_id)
}

fn effective_material_size(config: &AppConfig, height: i32) -> u32 {
    let available = (height - MATERIAL_VERTICAL_PADDING).max(1) as u32;
    u32::from(config.widget_visual.material_display_size_px).min(available)
}

fn group_track_width(config: &AppConfig, id: WidgetGroupId, height: i32) -> i32 {
    if selected_material_group(config, id).is_some() {
        let size = effective_material_size(config, height) as i32;
        size * 3 + MATERIAL_GAP * 2
    } else {
        LAMP_TRACK_WIDTH
    }
}

fn group_width_for(config: &AppConfig, id: WidgetGroupId, height: i32) -> i32 {
    widget_image::get_logo(id).width as i32 + LOGO_GAP + group_track_width(config, id, height)
}

/// Total widget width for the enabled sources, including margins between groups.
pub fn total_widget_width(config: &AppConfig, height: i32) -> i32 {
    let group_ids = visible_group_ids(config);
    group_ids
        .iter()
        .map(|id| group_width_for(config, *id, height))
        .sum::<i32>()
        + GROUP_MARGIN * (group_ids.len().saturating_sub(1) as i32)
}

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
    let material_brightness = MaterialBrightness::from_config(config);
    let groups = build_group_layouts(snapshot, effects, now_ms, config, width, height);

    for group in &groups {
        draw_group(&mut buffer, group, palette, material_brightness);
    }

    // Vertical divider between groups.
    // Drawn in the gap between cells, with configurable width and height.
    if groups.len() > 1 {
        let divider_h = (height * DIVIDER_HEIGHT_PERCENT as i32) / 100;
        let divider_top = (height - divider_h) / 2;
        let divider_bottom = divider_top + divider_h;

        for group in groups.iter().take(groups.len() - 1) {
            let gap_center = group.cell_right + GROUP_MARGIN / 2;
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
    visible_group_ids(config)
        .into_iter()
        .scan(0, |cell_left, id| {
            let cell_width = group_width_for(config, id, height);
            let current_left = *cell_left;
            *cell_left += cell_width + GROUP_MARGIN;
            Some((id, current_left, cell_width))
        })
        .map(|(id, cell_left, cell_width)| {
            let cell_right = (cell_left + cell_width).min(width);
            let logo_width = widget_image::get_logo(id).width as i32;
            let logo_h = widget_image::get_logo(id).height as i32;
            let material_size = effective_material_size(config, height);
            let has_material = selected_material_group(config, id).is_some();
            let group_height = if has_material {
                GROUP_HEIGHT.max(material_size as i32)
            } else {
                GROUP_HEIGHT
            };
            let group_top = ((height - group_height) / 2).max(0);
            let hot_zone = RECT {
                left: (cell_left + 4).max(0),
                top: (group_top - 2).max(0),
                right: (cell_right - 4).max((cell_left + GROUP_WIDTH).min(width)),
                bottom: (group_top + group_height + 2).min(height),
            };
            let lamp_center_y = height as f32 / 2.0;
            let first_center_x = cell_left as f32
                + logo_width as f32
                + LOGO_GAP as f32
                + if has_material {
                    material_size as f32 / 2.0
                } else {
                    LAMP_RADIUS
                };
            let lamp_spacing = if has_material {
                material_size as f32 + MATERIAL_GAP as f32
            } else {
                LAMP_SPACING as f32
            };

            GroupLayout {
                id,
                cell_right,
                logo_left: cell_left,
                logo_top: group_top + (group_height - logo_h) / 2,
                render_state: effects.render_state_for(snapshot, id.source_id(), now_ms),
                hot_zone,
                lights: [
                    LampLayout {
                        center_x: first_center_x,
                        center_y: lamp_center_y,
                        radius: LAMP_RADIUS,
                    },
                    LampLayout {
                        center_x: first_center_x + lamp_spacing,
                        center_y: lamp_center_y,
                        radius: LAMP_RADIUS,
                    },
                    LampLayout {
                        center_x: first_center_x + lamp_spacing * 2.0,
                        center_y: lamp_center_y,
                        radius: LAMP_RADIUS,
                    },
                ],
                material: selected_material_images(config, id, material_size),
            }
        })
        .collect()
}

fn draw_group(
    buffer: &mut PixelBuffer,
    group: &GroupLayout,
    palette: Palette,
    material_brightness: MaterialBrightness,
) {
    let logo = widget_image::get_logo(group.id);
    widget_image::blit_logo(buffer, group.logo_left, group.logo_top, logo);

    if let Some(material) = &group.material {
        draw_material_lamps(buffer, group, material, material_brightness);
        return;
    }

    for (index, lamp) in group.lights.iter().enumerate() {
        let lamp_state = group.render_state.lamps[index];
        let recipe = lamp_paint_recipe(index, palette);
        draw_idle_lamp(buffer, lamp, recipe);

        if lamp_state.alpha > 0 {
            draw_active_lamp(buffer, lamp, recipe, lamp_state.alpha);
        }
    }
}

fn selected_material_images(
    config: &AppConfig,
    id: WidgetGroupId,
    size: u32,
) -> Option<Arc<widget_image::MaterialGroupImages>> {
    widget_image::get_material_group_images(selected_material_group(config, id)?, size).ok()
}

fn draw_material_lamps(
    buffer: &mut PixelBuffer,
    group: &GroupLayout,
    material: &widget_image::MaterialGroupImages,
    brightness: MaterialBrightness,
) {
    let images = [&material.green, &material.yellow, &material.red];

    for (index, lamp) in group.lights.iter().enumerate() {
        let image = images[index];
        let left = (lamp.center_x - image.width as f32 / 2.0).round() as i32;
        let top = (lamp.center_y - image.height as f32 / 2.0).round() as i32;
        let opacity = material_opacity(group.render_state, index, brightness);
        widget_image::blit_material(buffer, left, top, image, opacity);
    }
}

fn material_opacity(state: GroupRenderState, index: usize, brightness: MaterialBrightness) -> u8 {
    let percent = match state.display_state {
        crate::ui_state::SourceVisualState::Idle => brightness.idle_percent,
        crate::ui_state::SourceVisualState::Completed => {
            if index == 0 {
                brightness.steady_percent
            } else {
                brightness.idle_percent
            }
        }
        crate::ui_state::SourceVisualState::Working
        | crate::ui_state::SourceVisualState::NeedsAttention
        | crate::ui_state::SourceVisualState::Error => {
            if state.lamps[index].alpha == u8::MAX {
                brightness.blink_percent
            } else {
                brightness.idle_percent
            }
        }
    };
    ((u16::from(percent) * 255) / 100) as u8
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

impl MaterialBrightness {
    fn from_config(config: &AppConfig) -> Self {
        let idle_percent = config
            .widget_visual
            .material_idle_brightness_percent
            .min(80);
        Self {
            idle_percent,
            blink_percent: config
                .widget_visual
                .material_blink_brightness_percent
                .clamp(idle_percent, 100),
            steady_percent: config
                .widget_visual
                .material_steady_brightness_percent
                .clamp(idle_percent, 100),
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
        app_config::{AppConfig, MaterialGroup},
        ui_state::{AppStatusSnapshot, SourceVisualState},
        widget_effects::{LampRenderState, WidgetEffectsState},
    };
    use image::{ImageBuffer, RgbaImage};
    use std::{env, fs, path::Path};

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

    #[test]
    fn material_brightness_uses_final_opacity_for_each_effect_state() {
        let brightness = MaterialBrightness {
            idle_percent: 42,
            blink_percent: 100,
            steady_percent: 84,
        };
        let idle = GroupRenderState {
            display_state: SourceVisualState::Idle,
            lamps: [LampRenderState { alpha: 0 }; 3],
        };
        let blink_on = GroupRenderState {
            display_state: SourceVisualState::Working,
            lamps: [
                LampRenderState { alpha: u8::MAX },
                LampRenderState { alpha: 0 },
                LampRenderState { alpha: 0 },
            ],
        };
        let blink_off = GroupRenderState {
            display_state: SourceVisualState::Working,
            lamps: [
                LampRenderState { alpha: 136 },
                LampRenderState { alpha: 0 },
                LampRenderState { alpha: 0 },
            ],
        };
        let completed = GroupRenderState {
            display_state: SourceVisualState::Completed,
            lamps: [
                LampRenderState { alpha: u8::MAX },
                LampRenderState { alpha: 0 },
                LampRenderState { alpha: 0 },
            ],
        };

        assert_eq!(material_opacity(idle, 0, brightness), 107);
        assert_eq!(material_opacity(blink_on, 0, brightness), 255);
        assert_eq!(material_opacity(blink_off, 0, brightness), 107);
        assert_eq!(material_opacity(completed, 0, brightness), 214);
        assert_eq!(material_opacity(completed, 1, brightness), 107);
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

    fn write_material_png(path: &Path, color: [u8; 4]) {
        let image: RgbaImage = ImageBuffer::from_pixel(4, 4, image::Rgba(color));
        image.save(path).expect("material test PNG should save");
    }

    #[test]
    fn custom_materials_preserve_state_alpha_and_replace_builtin_lamps() {
        let root = env::temp_dir().join(format!(
            "cc-traffic-light-render-material-{}",
            std::process::id()
        ));
        let _ = fs::remove_dir_all(&root);
        fs::create_dir_all(&root).expect("test directory should exist");
        write_material_png(&root.join("green.png"), [0, 0, 255, 255]);
        write_material_png(&root.join("yellow.png"), [255, 0, 255, 255]);
        write_material_png(&root.join("red.png"), [0, 255, 255, 255]);

        let group = MaterialGroup {
            id: format!("render-test-{}", std::process::id()),
            name: "Render test".to_string(),
            green_path: root.join("green.png").display().to_string(),
            yellow_path: root.join("yellow.png").display().to_string(),
            red_path: root.join("red.png").display().to_string(),
        };
        let layout = single_group_layout(SourceVisualState::Working);
        let frame = render_frame(GROUP_WIDTH, 32, SourceVisualState::Working, |config| {
            config.monitoring.claude_enabled = false;
            config.widget_visual.material_groups = vec![group.clone()];
            config.widget_visual.codex_material_group_id = Some(group.id.clone());
        });
        let active = lamp_center(&layout, 0);
        let inactive = lamp_center(&layout, 1);
        let active_pixel = pixel_rgba(&frame, active.0, active.1);
        let inactive_pixel = pixel_rgba(&frame, inactive.0, inactive.1);

        assert!(
            active_pixel.2 > active_pixel.0 && active_pixel.2 > active_pixel.1,
            "active green slot should use the supplied blue test material: {active_pixel:?}"
        );
        assert!(
            u16::from(inactive_pixel.0) + u16::from(inactive_pixel.1) + u16::from(inactive_pixel.2)
                < u16::from(active_pixel.0) + u16::from(active_pixel.1) + u16::from(active_pixel.2),
            "inactive material should remain dimmer than the active material"
        );
        assert_eq!(
            frame.hot_zones.len(),
            1,
            "custom material keeps the group hot zone"
        );

        let _ = fs::remove_dir_all(root);
    }

    #[test]
    fn custom_material_layout_uses_requested_size_and_caps_to_available_height() {
        let root = env::temp_dir().join(format!(
            "cc-traffic-light-material-layout-{}",
            std::process::id()
        ));
        let _ = fs::remove_dir_all(&root);
        fs::create_dir_all(&root).expect("test directory should exist");
        for (name, color) in [
            ("green.png", [0, 255, 0, 255]),
            ("yellow.png", [255, 255, 0, 255]),
            ("red.png", [255, 0, 0, 255]),
        ] {
            write_material_png(&root.join(name), color);
        }

        let group = MaterialGroup {
            id: format!("layout-test-{}", std::process::id()),
            name: "Layout test".to_string(),
            green_path: root.join("green.png").display().to_string(),
            yellow_path: root.join("yellow.png").display().to_string(),
            red_path: root.join("red.png").display().to_string(),
        };
        let mut config = AppConfig::default_v1();
        config.monitoring.claude_enabled = false;
        config.widget_visual.material_groups = vec![group.clone()];
        config.widget_visual.codex_material_group_id = Some(group.id);
        config.widget_visual.material_display_size_px = 32;

        let short_layout = build_group_layouts(
            &snapshot_with_state(SourceVisualState::Working),
            &WidgetEffectsState::default(),
            0,
            &config,
            200,
            24,
        );
        let short_group = &short_layout[0];
        assert_eq!(short_group.material.as_ref().unwrap().green.width, 16);
        assert_eq!(
            short_group.lights[1].center_x - short_group.lights[0].center_x,
            24.0
        );

        let tall_layout = build_group_layouts(
            &snapshot_with_state(SourceVisualState::Working),
            &WidgetEffectsState::default(),
            0,
            &config,
            300,
            40,
        );
        let tall_group = &tall_layout[0];
        assert_eq!(tall_group.material.as_ref().unwrap().green.width, 32);
        assert_eq!(
            tall_group.lights[1].center_x - tall_group.lights[0].center_x,
            40.0
        );
        assert!(tall_group.hot_zone.bottom - tall_group.hot_zone.top >= 32);

        let _ = fs::remove_dir_all(root);
    }
}
