//! Tema visual: modo oscuro, color de acento, esquinas y tamanos de texto.
use eframe::egui::{self, Color32, CornerRadius, FontId, TextStyle, Visuals};

pub const ACENTO: Color32 = Color32::from_rgb(0, 120, 190);

/// Se llama una vez al crear la ventana. No cambia la fuente.
pub fn aplicar_tema(ctx: &egui::Context) {
    ctx.set_theme(egui::Theme::Dark);
    ctx.all_styles_mut(|s| {
        let mut v = Visuals::dark();
        v.selection.bg_fill = ACENTO;
        v.window_corner_radius = CornerRadius::same(6);
        v.menu_corner_radius = CornerRadius::same(6);
        for w in [&mut v.widgets.noninteractive, &mut v.widgets.inactive, &mut v.widgets.hovered, &mut v.widgets.active, &mut v.widgets.open] {
            w.corner_radius = CornerRadius::same(6);
        }
        s.visuals = v;
        s.spacing.item_spacing = egui::vec2(8.0, 6.0);
        s.spacing.button_padding = egui::vec2(10.0, 5.0);
        s.spacing.interact_size.y = 26.0;
        s.text_styles.insert(TextStyle::Body, FontId::proportional(14.5));
        s.text_styles.insert(TextStyle::Button, FontId::proportional(14.5));
        s.text_styles.insert(TextStyle::Heading, FontId::proportional(19.0));
        s.text_styles.insert(TextStyle::Monospace, FontId::monospace(13.0));
    });
}
