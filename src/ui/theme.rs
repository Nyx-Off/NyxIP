use egui::{Color32, Visuals, Style, CornerRadius, Stroke, Shadow, Vec2};

pub const BG_DARK: Color32 = Color32::from_rgb(18, 18, 30);
pub const BG_PANEL: Color32 = Color32::from_rgb(25, 25, 45);
pub const BG_WIDGET: Color32 = Color32::from_rgb(35, 35, 60);
pub const BG_HOVER: Color32 = Color32::from_rgb(45, 45, 75);
pub const ACCENT_CYAN: Color32 = Color32::from_rgb(0, 210, 255);
pub const ACCENT_PURPLE: Color32 = Color32::from_rgb(150, 100, 255);
pub const TEXT_PRIMARY: Color32 = Color32::from_rgb(220, 220, 240);
pub const TEXT_DIM: Color32 = Color32::from_rgb(140, 140, 170);
pub const GREEN: Color32 = Color32::from_rgb(0, 230, 118);
pub const RED: Color32 = Color32::from_rgb(255, 82, 82);
pub const ORANGE: Color32 = Color32::from_rgb(255, 171, 64);

pub fn apply_theme(ctx: &egui::Context) {
    let mut style = Style::default();

    let mut visuals = Visuals::dark();
    visuals.override_text_color = Some(TEXT_PRIMARY);
    visuals.panel_fill = BG_DARK;
    visuals.window_fill = BG_PANEL;
    visuals.extreme_bg_color = BG_WIDGET;
    visuals.faint_bg_color = BG_PANEL;

    // Widgets
    visuals.widgets.noninteractive.bg_fill = BG_PANEL;
    visuals.widgets.noninteractive.fg_stroke = Stroke::new(1.0, TEXT_DIM);
    visuals.widgets.noninteractive.corner_radius = CornerRadius::same(6);

    visuals.widgets.inactive.bg_fill = BG_WIDGET;
    visuals.widgets.inactive.fg_stroke = Stroke::new(1.0, TEXT_PRIMARY);
    visuals.widgets.inactive.corner_radius = CornerRadius::same(6);

    visuals.widgets.hovered.bg_fill = BG_HOVER;
    visuals.widgets.hovered.fg_stroke = Stroke::new(1.5, ACCENT_CYAN);
    visuals.widgets.hovered.corner_radius = CornerRadius::same(6);

    visuals.widgets.active.bg_fill = ACCENT_CYAN;
    visuals.widgets.active.fg_stroke = Stroke::new(2.0, BG_DARK);
    visuals.widgets.active.corner_radius = CornerRadius::same(6);

    visuals.selection.bg_fill = Color32::from_rgba_premultiplied(0, 210, 255, 40);
    visuals.selection.stroke = Stroke::new(1.0, ACCENT_CYAN);

    visuals.window_shadow = Shadow {
        offset: [0, 4],
        blur: 12,
        spread: 0,
        color: Color32::from_black_alpha(80),
    };

    style.visuals = visuals;
    style.spacing.item_spacing = Vec2::new(8.0, 6.0);
    style.spacing.button_padding = Vec2::new(12.0, 6.0);

    ctx.set_style(style);
}
