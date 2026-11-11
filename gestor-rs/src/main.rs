mod ajustes;
mod app;
mod cfg;
mod servidor;
mod tema;

fn main() -> eframe::Result {
    let opciones = eframe::NativeOptions {
        viewport: eframe::egui::ViewportBuilder::default().with_inner_size([1240.0, 800.0]).with_title("Gestor de servidor MW3 LAN"),
        ..Default::default()
    };
    eframe::run_native("Gestor de servidor MW3 LAN", opciones, Box::new(|cc| {
        tema::aplicar_tema(&cc.egui_ctx);
        Ok(Box::new(app::App::nueva()))
    }))
}
