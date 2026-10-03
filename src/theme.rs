use eframe::egui::{self, Color32, Rounding, Stroke};

pub fn apply_theme(ctx: &egui::Context) {
    let mut visuals = egui::Visuals::dark();
    let rounding = Rounding::same(8.0);
    visuals.widgets.noninteractive.rounding = rounding;
    visuals.widgets.inactive.rounding = rounding;
    visuals.widgets.hovered.rounding = rounding;
    visuals.widgets.active.rounding = rounding;

    let accent = Color32::from_rgb(94, 129, 244);
    visuals.selection.bg_fill = accent;
    visuals.selection.stroke = Stroke::new(1.0_f32, accent);

    visuals.panel_fill = Color32::from_rgb(24, 24, 28);
    visuals.window_fill = Color32::from_rgb(24, 24, 28);

    ctx.set_visuals(visuals);

    let mut style = (*ctx.style()).clone();
    style.spacing.item_spacing = egui::vec2(10.0, 12.0);
    style.spacing.button_padding = egui::vec2(14.0, 8.0);
    style.spacing.window_margin = egui::Margin::same(16.0);
    ctx.set_style(style);
}
