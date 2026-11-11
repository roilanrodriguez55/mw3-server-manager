//! Ventana principal: perfiles .cfg, formulario, texto del archivo, servidor y consola. Portada de gestor_servidor.py.
use crate::ajustes::{self, Ajustes};
use crate::cfg::{self, Bots, Form, BASE, EXTRA, MODOS, PLAY};
use crate::servidor::{self, Servidor};
use crate::tema::ACENTO;
use eframe::egui::{self, Color32};
use std::fs;
use std::path::{Path, PathBuf};
use std::time::{Duration, Instant};

const VERDE: Color32 = Color32::from_rgb(0x3f, 0xb9, 0x50);
const VERDE_BOTON: Color32 = Color32::from_rgb(0x2a, 0x6b, 0x3c);
const ROJO_BOTON: Color32 = Color32::from_rgb(0x8a, 0x3a, 0x3a);
const ROJO_TEXTO: Color32 = Color32::from_rgb(0xe8, 0x9a, 0x9a);
const NARANJA: Color32 = Color32::from_rgb(0xe8, 0x9a, 0x2c);
const GRIS: Color32 = Color32::from_rgb(0x8a, 0x8f, 0x98);
/// Ancho de la etiqueta y del campo en las cuadriculas (dos tarjetas caben lado a lado).
const CAMPO_W: f32 = 200.0;
const NOMBRE_MAL: &str = "El nombre del archivo solo admite letras, numeros, _ y -.";
const FILL_MODOS: &[&str] = &["0 cuenta a todos", "1 solo bots", "2 por equipo (mapas)", "3 por equipo con bots"];
const SKILLS: &[&str] = &[
    "0 aleatorio (entre min y max)", "1 facil", "2", "3", "4", "5", "6", "7 dificil", "8 personalizada", "9 totalmente aleatorio",
];
const EQUIPOS: &[&str] = &["autoassign", "allies", "axis", "spectator"];

#[derive(Clone, Copy, PartialEq)]
enum Pestana {
    General,
    Rotacion,
    Bots,
    Texto,
}

/// Lo que el usuario quiere hacer; si hay cambios sin guardar, primero se pregunta.
enum Accion {
    Ninguna,
    Nuevo,
    Perfil(String),
    AbrirOtro,
    Carpeta(PathBuf),
    Duplicar(String),
    Iniciar,
    Cerrar,
}

enum Dialogo {
    Mensaje(String),
    SinGuardar(Accion),
    Sobrescribir(PathBuf, Accion),
    Eliminar(PathBuf),
    Duplicar(String),
    Importar(Vec<(PathBuf, PathBuf)>),
    ServidorAbierto,
}

impl Dialogo {
    fn botones(&self) -> &'static [&'static str] {
        match self {
            Dialogo::Mensaje(_) => &["Aceptar"],
            Dialogo::SinGuardar(_) => &["Guardar", "Descartar", "Cancelar"],
            Dialogo::Sobrescribir(..) => &["Sobrescribir", "Cancelar"],
            Dialogo::Eliminar(_) => &["Eliminar", "Cancelar"],
            Dialogo::Duplicar(_) => &["Crear", "Cancelar"],
            Dialogo::Importar(_) => &["Sobrescribir", "Omitir", "Cancelar"],
            Dialogo::ServidorAbierto => &["Cerrar igual", "Cancelar"],
        }
    }

    fn titulo(&self) -> &'static str {
        match self {
            Dialogo::Mensaje(_) => "Aviso",
            Dialogo::SinGuardar(_) => "Cambios sin guardar",
            Dialogo::Sobrescribir(..) | Dialogo::Importar(_) => "Archivo existente",
            Dialogo::Eliminar(_) => "Eliminar perfil",
            Dialogo::Duplicar(_) => "Duplicar perfil",
            Dialogo::ServidorAbierto => "Servidor abierto",
        }
    }
}

pub struct App {
    aj: Ajustes,
    form: Form,
    texto: String,
    manual: bool,
    sucio: bool,
    archivo: Option<PathBuf>,
    archivos: Vec<String>,
    srv: Servidor,
    log: String,
    log_t: Instant,
    dialogo: Option<Dialogo>,
    ajustes_draft: Option<Ajustes>,
    instalaciones: Vec<String>,
    pestana: Pestana,
    sel_rot: Option<usize>,
    modo_sel: String,
    mapa_sel: String,
    cerrar: bool,
}

impl App {
    pub fn nueva() -> Self {
        let mut aj = ajustes::cargar();
        if aj.storage.is_empty() {
            aj.storage = ajustes::buscar_storage().into_iter().next().unwrap_or_default();
        }
        if aj.wine.is_empty() && !cfg!(windows) {
            aj.wine = ajustes::buscar_wine();
        }
        let mut app = App {
            aj,
            form: Form::default(),
            texto: String::new(),
            manual: false,
            sucio: false,
            archivo: None,
            archivos: Vec::new(),
            srv: Servidor::default(),
            log: String::new(),
            log_t: Instant::now(),
            dialogo: None,
            ajustes_draft: None,
            instalaciones: Vec::new(),
            pestana: Pestana::General,
            sel_rot: None,
            modo_sel: "war".into(),
            mapa_sel: "mp_seatown".into(),
            cerrar: false,
        };
        app.listar();
        let ruta = app.dir().join(&app.aj.ultimo);
        if ruta.is_file() {
            app.abrir(&ruta);
        } else {
            app.texto = cfg::construir(&app.form);
        }
        app
    }

    fn dir(&self) -> PathBuf {
        PathBuf::from(&self.aj.perfiles)
    }

    fn ruta(&self, nombre: &str) -> PathBuf {
        self.dir().join(format!("mw3_{nombre}.cfg"))
    }

    fn listar(&mut self) {
        let mut nombres: Vec<String> = fs::read_dir(self.dir())
            .map(|rd| rd.flatten().map(|e| e.file_name().to_string_lossy().into_owned()).collect())
            .unwrap_or_default();
        nombres.retain(|n| {
            let l = n.to_ascii_lowercase();
            l.ends_with(".cfg") && l != "server.cfg"
        });
        nombres.sort();
        self.archivos = nombres;
    }

    fn guardar_aj(&self) {
        let _ = ajustes::guardar(&self.aj);
    }

    fn aviso(&mut self, texto: impl Into<String>) {
        self.dialogo = Some(Dialogo::Mensaje(texto.into()));
    }

    /// Cambio en el formulario: el texto se regenera salvo que se haya editado a mano.
    fn form_cambio(&mut self) {
        self.sucio = true;
        if !self.manual {
            self.texto = cfg::construir(&self.form);
        }
    }

    fn pedir(&mut self, a: Accion) {
        if self.sucio {
            self.dialogo = Some(Dialogo::SinGuardar(a));
        } else {
            self.ejecutar(a);
        }
    }

    fn ejecutar(&mut self, a: Accion) {
        match a {
            Accion::Ninguna => {}
            Accion::Nuevo => self.nuevo(),
            Accion::Perfil(n) => {
                let ruta = self.dir().join(n);
                self.abrir(&ruta);
            }
            Accion::AbrirOtro => {
                if let Some(ruta) = rfd::FileDialog::new().add_filter("Configuracion", &["cfg"]).pick_file() {
                    self.abrir(&ruta);
                }
            }
            Accion::Carpeta(d) => {
                self.aj.perfiles = d.to_string_lossy().into_owned();
                self.archivo = None;
                self.guardar_aj();
                self.listar();
            }
            Accion::Duplicar(n) => {
                let dst = self.ruta(&n);
                if dst.exists() {
                    self.aviso(format!("{} ya existe.", nombre_archivo(&dst)));
                } else if let Some(src) = self.archivo.clone() {
                    match fs::copy(&src, &dst) {
                        Ok(_) => self.abrir(&dst),
                        Err(e) => self.aviso(format!("No se pudo duplicar: {e}")),
                    }
                }
            }
            Accion::Iniciar => self.arrancar(),
            Accion::Cerrar => {
                if self.srv.corriendo() {
                    self.dialogo = Some(Dialogo::ServidorAbierto);
                } else {
                    self.cerrar_ya();
                }
            }
        }
    }

    fn cerrar_ya(&mut self) {
        self.guardar_aj();
        self.cerrar = true;
    }

    fn validar(&self) -> Result<(), String> {
        if !nombre_valido(&self.form.nombre) {
            return Err(NOMBRE_MAL.into());
        }
        if self.manual { Ok(()) } else { cfg::validar(&self.form) }
    }

    fn intentar_guardar(&mut self, despues: Accion) {
        if let Err(e) = self.validar() {
            self.aviso(e);
            return;
        }
        let destino = self.ruta(&self.form.nombre);
        if self.archivo.as_ref() != Some(&destino) && destino.exists() {
            self.dialogo = Some(Dialogo::Sobrescribir(destino, despues));
        } else {
            self.escribir(destino, despues);
        }
    }

    fn escribir(&mut self, destino: PathBuf, despues: Accion) {
        let contenido = format!("{}\n", self.texto.trim_end_matches('\n'));
        if let Err(e) = fs::write(&destino, contenido) {
            self.aviso(format!("No se pudo guardar: {e}"));
            return;
        }
        self.aj.ultimo = nombre_archivo(&destino);
        self.archivo = Some(destino);
        self.sucio = false;
        self.listar();
        self.guardar_aj();
        self.ejecutar(despues);
    }

    fn abrir(&mut self, ruta: &Path) {
        let texto = match fs::read(ruta) {
            Ok(b) => String::from_utf8_lossy(&b).into_owned(),
            Err(e) => {
                self.aviso(format!("No se pudo abrir: {e}"));
                return;
            }
        };
        let nombre = nombre_archivo(ruta);
        let base = nombre.strip_suffix(".cfg").unwrap_or(&nombre);
        let mut form = cfg::parsear(&texto);
        form.nombre = base.strip_prefix("mw3_").unwrap_or(base).to_string();
        self.form = form;
        self.texto = texto;
        self.manual = false;
        self.sucio = false;
        self.aj.ultimo = nombre;
        self.archivo = Some(ruta.to_path_buf());
        self.listar();
    }

    fn nuevo(&mut self) {
        self.archivo = None;
        self.form = Form {
            rotacion: vec![("war".into(), "mp_seatown".into()), ("war".into(), "mp_dome".into())],
            ..Form::default()
        };
        self.texto = cfg::construir(&self.form);
        self.manual = false;
        self.sucio = true;
    }

    fn pedir_duplicar(&mut self) {
        if self.archivo.is_some() {
            self.dialogo = Some(Dialogo::Duplicar(String::new()));
        } else {
            self.aviso("Elige primero un perfil.");
        }
    }

    /// El perfil seleccionado en la lista, si esta dentro de la carpeta de perfiles.
    fn seleccion_eliminable(&self) -> Option<PathBuf> {
        let p = self.archivo.clone()?;
        let en_lista = self.archivos.contains(&nombre_archivo(&p));
        (en_lista && p.parent() == Some(self.dir().as_path())).then_some(p)
    }

    fn eliminar(&mut self, p: PathBuf) {
        match fs::remove_file(&p) {
            Ok(()) => {
                if self.archivo.as_ref() == Some(&p) {
                    self.archivo = None;
                    self.sucio = true;
                }
                self.listar();
            }
            Err(e) => self.aviso(format!("No se pudo eliminar: {e}")),
        }
    }

    fn elegir_carpeta(&mut self) {
        if let Some(d) = rfd::FileDialog::new().set_directory(&self.aj.perfiles).pick_folder() {
            self.pedir(Accion::Carpeta(d));
        }
    }

    fn importar(&mut self) {
        let Some(rutas) = rfd::FileDialog::new()
            .set_directory(&self.aj.perfiles)
            .add_filter("Configuracion", &["cfg"])
            .pick_files()
        else {
            return;
        };
        let mut pendientes = Vec::new();
        for src in rutas {
            let Some(nombre) = src.file_name() else { continue };
            let dst = self.dir().join(nombre);
            if dst.exists() {
                pendientes.push((src, dst));
            } else if let Err(e) = fs::copy(&src, &dst) {
                self.aviso(format!("No se pudo copiar {}: {e}", nombre_archivo(&src)));
            }
        }
        if !pendientes.is_empty() && self.dialogo.is_none() {
            self.dialogo = Some(Dialogo::Importar(pendientes));
        }
        self.listar();
    }

    /// Respuesta a la cola de importacion: 0 sobrescribe, 1 omite, 2 cancela el resto.
    fn seguir_importando(&mut self, mut pares: Vec<(PathBuf, PathBuf)>, i: usize) {
        if i == 2 {
            pares.clear();
        } else {
            let (src, dst) = pares.remove(0);
            if i == 0 && let Err(e) = fs::copy(&src, &dst) {
                pares.clear();
                self.aviso(format!("No se pudo copiar {}: {e}", nombre_archivo(&src)));
            }
        }
        if !pares.is_empty() {
            self.dialogo = Some(Dialogo::Importar(pares));
        }
        self.listar();
    }

    fn arrancar(&mut self) {
        if self.srv.corriendo() {
            self.aviso("Ya hay un servidor iniciado desde esta ventana. Detenlo primero.");
            return;
        }
        let Some(ruta) = self.archivo.clone() else {
            self.aviso("Elige o guarda un perfil primero.");
            return;
        };
        let puerto = self.aj.puerto.trim().to_string();
        if !servidor::puerto_valido(&puerto) {
            self.aviso(servidor::PUERTO_MAL);
            return;
        }
        if !servidor::puerto_libre(&puerto) {
            self.aviso(format!("El puerto UDP {puerto} ya esta en uso: hay otro servidor abierto. Cierralo o usa otro puerto."));
            return;
        }
        match self.srv.iniciar(&self.aj, &ruta, &puerto) {
            Err(e) => self.aviso(e),
            Ok(()) => {
                self.aj.puerto = puerto;
                self.guardar_aj();
            }
        }
    }

    fn responder(&mut self, d: Dialogo, i: usize) {
        match (d, i) {
            (Dialogo::SinGuardar(a), 0) => self.intentar_guardar(a),
            (Dialogo::SinGuardar(a), 1) => {
                self.sucio = false;
                self.ejecutar(a);
            }
            (Dialogo::Sobrescribir(p, a), 0) => self.escribir(p, a),
            (Dialogo::Eliminar(p), 0) => self.eliminar(p),
            (Dialogo::Duplicar(n), 0) => {
                if nombre_valido(&n) {
                    self.pedir(Accion::Duplicar(n));
                } else {
                    self.aviso(NOMBRE_MAL);
                }
            }
            (Dialogo::Importar(pares), _) => self.seguir_importando(pares, i),
            (Dialogo::ServidorAbierto, 0) => self.cerrar_ya(),
            _ => {}
        }
    }

    fn dialogos(&mut self, ctx: &egui::Context) {
        let Some(mut d) = self.dialogo.take() else { return };
        let botones = d.botones();
        let titulo = d.titulo();
        let principal = if matches!(d, Dialogo::Eliminar(_)) { ROJO_BOTON } else { ACENTO };
        let mut pulsado: Option<usize> = None;
        egui::Modal::new(egui::Id::unique("dialogo")).show(ctx, |ui| {
            ui.set_min_width(380.0);
            ui.heading(titulo);
            ui.add_space(6.0);
            match &mut d {
                Dialogo::Mensaje(t) => {
                    ui.label(t.as_str());
                }
                Dialogo::SinGuardar(_) => {
                    ui.label("Hay cambios sin guardar en el perfil actual.");
                }
                Dialogo::Sobrescribir(p, _) => {
                    ui.label(format!("{} ya existe. Sobrescribir?", nombre_archivo(p)));
                }
                Dialogo::Eliminar(p) => {
                    ui.label(format!("Eliminar {}? No se puede deshacer.", nombre_archivo(p)));
                }
                Dialogo::Duplicar(n) => {
                    ui.label("Nombre del nuevo perfil (sin mw3_ ni .cfg):");
                    ui.text_edit_singleline(n);
                }
                Dialogo::Importar(pares) => {
                    if let Some((_, dst)) = pares.first() {
                        ui.label(format!("{} ya existe. Sobrescribir?", nombre_archivo(dst)));
                    }
                }
                Dialogo::ServidorAbierto => {
                    ui.label("El servidor sigue abierto y seguira corriendo al cerrar esta ventana. Cerrar igual?");
                }
            }
            ui.add_space(12.0);
            ui.horizontal(|ui| {
                for (i, t) in botones.iter().enumerate() {
                    let boton = if i == 0 {
                        egui::Button::new(egui::RichText::new(*t).color(Color32::WHITE)).fill(principal)
                    } else {
                        egui::Button::new(*t)
                    };
                    if ui.add(boton.min_size(egui::vec2(96.0, 28.0))).clicked() {
                        pulsado = Some(i);
                    }
                }
            });
        });
        match pulsado {
            Some(i) => self.responder(d, i),
            None => self.dialogo = Some(d),
        }
    }

    fn ventana_ajustes(&mut self, ctx: &egui::Context) {
        let Some(mut d) = self.ajustes_draft.take() else { return };
        let mut abierta = true;
        let mut fin: Option<bool> = None;
        egui::Window::new("Ajustes de rutas")
            .open(&mut abierta)
            .collapsible(false)
            .min_width(380.0)
            .default_width(560.0)
            .show(ctx, |ui| {
                ruta_fila(ui, "Carpeta storage/iw5 de Plutonium", &mut d.storage, true);
                ui.horizontal(|ui| {
                    egui::ComboBox::from_id_salt("instalaciones").selected_text(d.storage.clone()).show_ui(ui, |ui| {
                        for r in &self.instalaciones {
                            ui.selectable_value(&mut d.storage, r.clone(), r.as_str());
                        }
                    });
                    if ui.button("Buscar instalaciones de Plutonium").clicked() {
                        self.instalaciones = ajustes::buscar_storage();
                    }
                });
                ruta_fila(ui, "Carpeta de MW3 (vacio = la de Plutonium)", &mut d.mw3, true);
                if !cfg!(windows) {
                    ruta_fila(ui, "Programa wine", &mut d.wine, false);
                }
                ui.add_space(8.0);
                ui.horizontal(|ui| {
                    let guardar = egui::Button::new(egui::RichText::new("Guardar").color(Color32::WHITE))
                        .fill(ACENTO)
                        .min_size(egui::vec2(96.0, 28.0));
                    if ui.add(guardar).clicked() {
                        fin = Some(true);
                    }
                    if ui.add(egui::Button::new("Cancelar").min_size(egui::vec2(96.0, 28.0))).clicked() {
                        fin = Some(false);
                    }
                });
            });
        match fin {
            Some(true) => {
                self.aj.storage = d.storage.trim().to_string();
                self.aj.mw3 = d.mw3.trim().to_string();
                self.aj.wine = d.wine.trim().to_string();
                self.guardar_aj();
            }
            None if abierta => self.ajustes_draft = Some(d),
            _ => {}
        }
    }

    fn panel_izq(&mut self, ui: &mut egui::Ui) {
        ui.horizontal(|ui| {
            ui.heading("Perfiles");
            if self.sucio {
                ui.colored_label(NARANJA, "●").on_hover_text("Cambios sin guardar");
            }
        });
        ui.add(egui::Label::new(egui::RichText::new(self.aj.perfiles.as_str()).weak()).truncate())
            .on_hover_text(self.aj.perfiles.as_str());
        ui.add_space(4.0);
        let ancho = (ui.available_width() - 6.0) / 2.0;
        let tam = egui::vec2(ancho, 28.0);
        egui::Grid::new("perfiles_acciones").num_columns(2).spacing([6.0, 6.0]).show(ui, |ui| {
            if ui.add_sized(tam, egui::Button::new("Nuevo")).clicked() {
                self.pedir(Accion::Nuevo);
            }
            if ui.add_sized(tam, egui::Button::new("Importar...")).clicked() {
                self.importar();
            }
            ui.end_row();
            if ui.add_sized(tam, egui::Button::new("Abrir...")).clicked() {
                self.pedir(Accion::AbrirOtro);
            }
            if ui.add_sized(tam, egui::Button::new("Duplicar")).clicked() {
                self.pedir_duplicar();
            }
            ui.end_row();
            if ui.add_sized(tam, egui::Button::new(egui::RichText::new("Eliminar").color(ROJO_TEXTO))).clicked() {
                match self.seleccion_eliminable() {
                    Some(p) => self.dialogo = Some(Dialogo::Eliminar(p)),
                    None => self.aviso("Selecciona un perfil de la lista para eliminarlo."),
                }
            }
            if ui.add_sized(tam, egui::Button::new("Carpeta...")).clicked() {
                self.elegir_carpeta();
            }
            ui.end_row();
        });
        ui.add_space(4.0);
        egui::Panel::bottom("servidor").show(ui, |ui| self.grupo_servidor(ui));
        let abierto = self.archivo.as_ref().map(|p| nombre_archivo(p));
        let mut elegido: Option<String> = None;
        egui::CentralPanel::default().show(ui, |ui| {
            let alto = ui.available_height() - 24.0;
            let fuente = egui::TextStyle::Body.resolve(ui.style());
            let color = ui.visuals().text_color();
            egui::ScrollArea::vertical().max_height(alto).auto_shrink([false, false]).show(ui, |ui| {
                ui.with_layout(egui::Layout::top_down_justified(egui::Align::LEFT), |ui| {
                    for n in &self.archivos {
                        let mut job = egui::text::LayoutJob::default();
                        if self.srv.perfil.as_deref() == Some(n.as_str()) {
                            job.append("● ", 0.0, egui::text::TextFormat { font_id: fuente.clone(), color: VERDE, ..Default::default() });
                        }
                        job.append(nombre_corto(n), 0.0, egui::text::TextFormat { font_id: fuente.clone(), color, ..Default::default() });
                        let fila = egui::Button::selectable(abierto.as_deref() == Some(n.as_str()), job)
                            .truncate()
                            .min_size(egui::vec2(ui.available_width(), 26.0));
                        if ui.add(fila).on_hover_text(n.as_str()).clicked() {
                            elegido = Some(n.clone());
                        }
                    }
                });
            });
            ui.weak(format!("{} perfiles", self.archivos.len()));
        });
        if let Some(n) = elegido.filter(|n| abierto.as_deref() != Some(n.as_str())) {
            self.pedir(Accion::Perfil(n));
        }
    }

    fn grupo_servidor(&mut self, ui: &mut egui::Ui) {
        let corre = self.srv.corriendo();
        egui::Frame::group(ui.style()).show(ui, |ui| {
            ui.strong("Servidor");
            ui.horizontal(|ui| {
                let (rect, _) = ui.allocate_exact_size(egui::vec2(10.0, 10.0), egui::Sense::hover());
                ui.painter().circle_filled(rect.center(), 5.0, if corre { VERDE } else { GRIS });
                if corre {
                    let texto = format!("En marcha: {} (puerto {})", self.srv.perfil.as_deref().unwrap_or(""), self.aj.puerto);
                    ui.add(egui::Label::new(texto).truncate());
                } else {
                    ui.weak("Detenido");
                }
            });
            ui.horizontal(|ui| {
                ui.label("Puerto");
                ui.add(egui::TextEdit::singleline(&mut self.aj.puerto).desired_width(80.0));
            });
            let ancho = ui.available_width();
            let iniciar = egui::Button::new(egui::RichText::new("Iniciar servidor").color(Color32::WHITE))
                .fill(VERDE_BOTON)
                .min_size(egui::vec2(ancho, 32.0));
            if ui.add_enabled(!corre, iniciar).on_hover_text("Inicia el servidor con el perfil seleccionado en la lista").clicked() {
                self.pedir(Accion::Iniciar);
            }
            let detener = egui::Button::new(egui::RichText::new("Detener servidor").color(Color32::WHITE))
                .fill(ROJO_BOTON)
                .min_size(egui::vec2(ancho, 32.0));
            if ui.add_enabled(corre, detener).clicked() && let Err(e) = self.srv.detener() {
                self.aviso(e);
            }
            if ui.add(egui::Button::new("Ajustes de rutas...").min_size(egui::vec2(ancho, 28.0))).clicked() {
                self.ajustes_draft = Some(self.aj.clone());
            }
        });
    }

    fn central(&mut self, ui: &mut egui::Ui) {
        let mut cambio = false;
        ui.horizontal(|ui| {
            ui.label("Archivo:");
            cambio |= ui.add(egui::TextEdit::singleline(&mut self.form.nombre).desired_width(220.0)).changed();
            ui.weak(".cfg  (se guarda como mw3_<nombre>.cfg)");
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                let guardar = egui::Button::new(egui::RichText::new("Guardar").color(Color32::WHITE))
                    .fill(ACENTO)
                    .min_size(egui::vec2(96.0, 28.0));
                if ui.add(guardar).clicked() {
                    self.intentar_guardar(Accion::Ninguna);
                }
                if self.sucio {
                    ui.colored_label(NARANJA, "Cambios sin guardar");
                }
            });
        });
        ui.separator();
        if self.manual {
            ui.colored_label(NARANJA, "Texto editado a mano: se guarda tal cual. Usa 'Leer el texto' para pasarlo al formulario.");
        }
        ui.horizontal(|ui| {
            ui.spacing_mut().button_padding.x = 14.0;
            for (p, t) in [
                (Pestana::General, "General"),
                (Pestana::Rotacion, "Rotacion"),
                (Pestana::Bots, "Bots"),
                (Pestana::Texto, "Texto del archivo"),
            ] {
                ui.selectable_value(&mut self.pestana, p, t);
            }
        });
        ui.separator();
        // Rotacion y Texto usan el alto restante con sus propios ScrollArea; General y Bots scrollean enteros.
        cambio |= match self.pestana {
            Pestana::General => egui::ScrollArea::vertical().auto_shrink([false, false]).show(ui, |ui| self.tab_general(ui)).inner,
            Pestana::Rotacion => self.tab_rotacion(ui),
            Pestana::Bots => egui::ScrollArea::vertical().auto_shrink([false, false]).show(ui, |ui| self.tab_bots(ui)).inner,
            Pestana::Texto => self.tab_texto(ui),
        };
        if cambio {
            self.form_cambio();
        }
    }

    fn tab_general(&mut self, ui: &mut egui::Ui) -> bool {
        let f = &mut self.form;
        let mut c = false;
        if ui.available_width() > 900.0 {
            ui.columns(2, |cols| {
                c |= gen_servidor(&mut cols[0], f);
                c |= gen_partida(&mut cols[1], f);
                c |= gen_reglas(&mut cols[1], f);
            });
        } else {
            c |= gen_servidor(ui, f);
            c |= gen_partida(ui, f);
            c |= gen_reglas(ui, f);
        }
        c
    }

    fn tab_rotacion(&mut self, ui: &mut egui::Ui) -> bool {
        let mapas: Vec<(&str, &str)> = BASE.iter().chain(EXTRA).copied().collect();
        let mut c = false;
        ui.horizontal(|ui| {
            egui::ComboBox::from_id_salt("modo").width(220.0).selected_text(etiqueta(MODOS, &self.modo_sel)).show_ui(ui, |ui| {
                for &(cod, nom) in MODOS {
                    ui.selectable_value(&mut self.modo_sel, cod.to_string(), format!("{nom} ({cod})"));
                }
            });
            egui::ComboBox::from_id_salt("mapa").width(260.0).selected_text(etiqueta(&mapas, &self.mapa_sel)).show_ui(ui, |ui| {
                for &(cod, nom) in &mapas {
                    ui.selectable_value(&mut self.mapa_sel, cod.to_string(), format!("{nom} ({cod})"));
                }
            });
            if ui.button("Agregar").clicked() {
                self.form.rotacion.push((self.modo_sel.clone(), self.mapa_sel.clone()));
                c = true;
            }
        });
        ui.add_space(4.0);
        ui.horizontal_wrapped(|ui| {
            if ui.button("Todos los mapas base con este modo").clicked() {
                self.agregar_varios(BASE);
                c = true;
            }
            if ui.button("Solo mapas Plutonium con este modo").clicked() {
                self.agregar_varios(EXTRA);
                c = true;
            }
            ui.add_space(12.0);
            if ui.button("Subir").clicked() && let Some(i) = self.sel_rot.filter(|&i| i > 0) {
                self.form.rotacion.swap(i, i - 1);
                self.sel_rot = Some(i - 1);
                c = true;
            }
            if ui.button("Bajar").clicked() && let Some(i) = self.sel_rot.filter(|&i| i + 1 < self.form.rotacion.len()) {
                self.form.rotacion.swap(i, i + 1);
                self.sel_rot = Some(i + 1);
                c = true;
            }
            if ui.button("Quitar").clicked() && let Some(i) = self.sel_rot.take() {
                self.form.rotacion.remove(i);
                c = true;
            }
            if ui.button("Vaciar").clicked() {
                self.form.rotacion.clear();
                self.sel_rot = None;
                c = true;
            }
        });
        ui.add_space(6.0);
        let alto = ui.available_height() - 40.0;
        egui::Frame::group(ui.style()).show(ui, |ui| {
            egui::ScrollArea::vertical().max_height(alto - 12.0).auto_shrink([false, false]).show(ui, |ui| {
                ui.with_layout(egui::Layout::top_down_justified(egui::Align::LEFT), |ui| {
                    let fuente = egui::TextStyle::Body.resolve(ui.style());
                    let color = ui.visuals().text_color();
                    for (i, (modo, mapa)) in self.form.rotacion.iter().enumerate() {
                        let mut job = egui::text::LayoutJob::default();
                        job.append(
                            &format!("{:>2}  ", i + 1),
                            0.0,
                            egui::text::TextFormat { font_id: egui::FontId::monospace(13.0), color: GRIS, ..Default::default() },
                        );
                        job.append(
                            &format!("{} - {}", nombre(MODOS, modo), nombre(&mapas, mapa)),
                            0.0,
                            egui::text::TextFormat { font_id: fuente.clone(), color, ..Default::default() },
                        );
                        let fila = egui::Button::selectable(self.sel_rot == Some(i), job)
                            .truncate()
                            .min_size(egui::vec2(ui.available_width(), 24.0));
                        if ui.add(fila).clicked() {
                            self.sel_rot = Some(i);
                        }
                    }
                });
            });
        });
        ui.weak("Al llegar al final vuelve a empezar. Los mapas Plutonium los descarga Plutonium aparte; los demas jugadores los necesitan.");
        c
    }

    fn agregar_varios(&mut self, lista: &[(&str, &str)]) {
        for (cod, _) in lista {
            self.form.rotacion.push((self.modo_sel.clone(), cod.to_string()));
        }
    }

    fn tab_bots(&mut self, ui: &mut egui::Ui) -> bool {
        let f = &mut self.form;
        let mut c = false;
        if ui.available_width() > 900.0 {
            ui.columns(2, |cols| {
                c |= bots_cantidad(&mut cols[0], f);
                c |= bots_dificultad(&mut cols[1], f);
            });
        } else {
            c |= bots_cantidad(ui, f);
            c |= bots_dificultad(ui, f);
        }
        c |= tarjeta(ui, "Comportamiento", "Desmarcar = el bot no lo hace.", |ui| {
            let mut c = false;
            let ancho = (ui.available_width() - 36.0) / 4.0;
            egui::Grid::new("bots_comportamiento")
                .num_columns(4)
                .spacing([12.0, 6.0])
                .min_col_width(ancho)
                .max_col_width(ancho)
                .show(ui, |ui| {
                    for (i, n) in PLAY.iter().enumerate() {
                        c |= ui.checkbox(&mut f.play[i], *n).changed();
                        if i % 4 == 3 {
                            ui.end_row();
                        }
                    }
                });
            c
        });
        c
    }

    fn tab_texto(&mut self, ui: &mut egui::Ui) -> bool {
        ui.label("Lineas extra (cualquier otra variable que quieras; se agregan al final):");
        let c = ui
            .add(egui::TextEdit::multiline(&mut self.form.extras).desired_rows(4).desired_width(f32::INFINITY))
            .changed();
        ui.horizontal(|ui| {
            if ui.button("Regenerar texto desde el formulario").clicked() {
                self.manual = false;
                self.texto = cfg::construir(&self.form);
                self.sucio = true;
            }
            if ui.button("Leer el texto al formulario").clicked() {
                let nombre = self.form.nombre.clone();
                let mut f = cfg::parsear(&self.texto);
                f.nombre = nombre;
                self.form = f;
                self.manual = false;
            }
        });
        ui.add_space(4.0);
        ui.label("Texto completo del archivo (editable a mano):");
        let alto = ui.available_height();
        let editado = egui::ScrollArea::vertical()
            .max_height(alto)
            .auto_shrink([false, false])
            .show(ui, |ui| {
                ui.add(
                    egui::TextEdit::multiline(&mut self.texto)
                        .code_editor()
                        .desired_width(f32::INFINITY)
                        .min_size(egui::vec2(0.0, alto - 12.0)),
                )
                .changed()
            })
            .inner;
        if editado {
            self.manual = true;
            self.sucio = true;
        }
        c
    }

    fn consola(&mut self, ui: &mut egui::Ui) {
        ui.strong("Consola de Plutonium (console.log)");
        egui::Frame::new().fill(ui.visuals().extreme_bg_color).inner_margin(6).show(ui, |ui| {
            egui::ScrollArea::both().auto_shrink([false, false]).stick_to_bottom(true).show(ui, |ui| {
                ui.add(egui::Label::new(egui::RichText::new(self.log.as_str()).monospace().size(12.0)).extend());
            });
        });
    }
}

impl eframe::App for App {
    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        let ctx = ui.ctx().clone();
        ctx.request_repaint_after(Duration::from_secs(1));
        if self.log_t.elapsed() >= Duration::from_secs(2) {
            self.log = servidor::leer_log(&self.aj.storage, 60);
            self.log_t = Instant::now();
        }
        if !self.cerrar && ctx.input(|i| i.viewport().close_requested()) {
            ctx.send_viewport_cmd(egui::ViewportCommand::CancelClose);
            self.pedir(Accion::Cerrar);
        }
        egui::Panel::left("perfiles")
            .default_size(280.0)
            .min_size(220.0)
            .resizable(true)
            .show(ui, |ui| self.panel_izq(ui));
        egui::Panel::bottom("consola")
            .default_size(170.0)
            .min_size(90.0)
            .resizable(true)
            .show(ui, |ui| self.consola(ui));
        egui::CentralPanel::default().show(ui, |ui| self.central(ui));
        self.ventana_ajustes(&ctx);
        self.dialogos(&ctx);
        if self.cerrar {
            ctx.send_viewport_cmd(egui::ViewportCommand::Close);
        }
    }
}

fn nombre_valido(n: &str) -> bool {
    !n.is_empty() && n.chars().all(|c| c.is_ascii_alphanumeric() || c == '_' || c == '-')
}

fn nombre_archivo(p: &Path) -> String {
    p.file_name().map_or_else(String::new, |n| n.to_string_lossy().into_owned())
}

/// Nombre de un perfil sin el prefijo mw3_ ni la extension .cfg.
fn nombre_corto(n: &str) -> &str {
    let base = n.strip_suffix(".cfg").unwrap_or(n);
    base.strip_prefix("mw3_").unwrap_or(base)
}

/// Nombre legible de un codigo de la lista (sin el codigo).
fn nombre(lista: &[(&str, &str)], cod: &str) -> String {
    lista.iter().find(|p| p.0 == cod).map_or_else(|| cod.to_string(), |p| p.1.to_string())
}

/// "Nombre (codigo)" del codigo de la lista.
fn etiqueta(lista: &[(&str, &str)], cod: &str) -> String {
    lista.iter().find(|p| p.0 == cod).map_or_else(|| cod.to_string(), |p| format!("{} ({})", p.1, p.0))
}

/// Primera palabra de una opcion ("7 dificil" -> "7").
fn token(opcion: &str) -> &str {
    opcion.split_whitespace().next().unwrap_or("")
}

/// Tarjeta con titulo en negrita y una nota gris opcional debajo.
fn tarjeta(ui: &mut egui::Ui, titulo: &str, nota: &str, contenido: impl FnOnce(&mut egui::Ui) -> bool) -> bool {
    egui::Frame::group(ui.style())
        .show(ui, |ui| {
            ui.strong(titulo);
            if !nota.is_empty() {
                ui.weak(nota);
            }
            ui.add_space(4.0);
            contenido(ui)
        })
        .inner
}

/// Cuadricula etiqueta | campo; ambas columnas miden lo mismo en todas las tarjetas.
fn rejilla(id: &str) -> egui::Grid {
    egui::Grid::new(id).num_columns(2).striped(true).spacing([12.0, 6.0]).min_col_width(CAMPO_W).max_col_width(CAMPO_W)
}

fn gen_servidor(ui: &mut egui::Ui, f: &mut Form) -> bool {
    tarjeta(ui, "Servidor", "", |ui| {
        let mut c = false;
        rejilla("gen_servidor").show(ui, |ui| {
            c |= fila_texto(ui, "Nombre en el listado", &mut f.hostname);
            c |= fila_texto(ui, "Maximo de jugadores (1-18)", &mut f.maxclients);
            c |= fila_texto(ui, "Contrasena (vacia = ninguna)", &mut f.password);
            c |= fila_texto(ui, "Contrasena rcon", &mut f.rcon);
            c |= fila_texto(ui, "Espacios privados", &mut f.privados);
            c |= fila_texto(ui, "Contrasena de espacios privados", &mut f.privpass);
            c |= fila_combo(ui, "Chat de voz (0 no, 1 libre, 2 equipo)", &mut f.voz, &["0", "1", "2"]);
            c |= fila_combo(ui, "Votaciones (0 no, 1 si)", &mut f.votos, &["0", "1"]);
            c |= fila_combo(ui, "Muertos hablan con vivos (0/1)", &mut f.chat_muertos, &["0", "1"]);
            c |= fila_texto(ui, "Segundos para echar por inactividad", &mut f.inactividad);
            c |= fila_texto(ui, "Segundos de baneo temporal", &mut f.ban);
            c |= fila_combo(ui, "Proteccion contra flood (0/1)", &mut f.flood, &["0", "1"]);
        });
        c
    })
}

fn gen_partida(ui: &mut egui::Ui, f: &mut Form) -> bool {
    tarjeta(ui, "Partida", "Vacio = el valor del juego; se aplica a todos los modos de la rotacion.", |ui| {
        let mut c = false;
        rejilla("gen_partida").show(ui, |ui| {
            c |= fila_texto(ui, "Minutos por partida (0 = sin limite)", &mut f.tiempo);
            c |= fila_texto(ui, "Limite de puntos", &mut f.puntos);
            c |= fila_texto(ui, "Rondas (SD, SAB, DD)", &mut f.rondas);
            c |= fila_texto(ui, "Rondas para ganar (SD, SAB, DD)", &mut f.ganar);
            c |= fila_texto(ui, "Vidas por ronda (0 = ilimitadas)", &mut f.vidas);
        });
        c
    })
}

fn gen_reglas(ui: &mut egui::Ui, f: &mut Form) -> bool {
    tarjeta(ui, "Reglas", "Nombres tomados del juego, efecto sin comprobar.", |ui| {
        let mut c = false;
        rejilla("gen_reglas").show(ui, |ui| {
            c |= fila_combo(ui, "Fuego amigo scr_team_fftype (0 no, 1 si, 2 reflejado, 3 compartido)", &mut f.fuego_amigo, &["", "0", "1", "2", "3"]);
            c |= fila_combo(ui, "Rachas scr_game_hardpoints (0 no, 1 si)", &mut f.rachas, &["", "0", "1"]);
            c |= fila_combo(ui, "Reaparicion forzada scr_player_forcerespawn", &mut f.respawn_forzado, &["", "0", "1"]);
        });
        c
    })
}

fn bots_cantidad(ui: &mut egui::Ui, f: &mut Form) -> bool {
    tarjeta(ui, "Cantidad", "", |ui| {
        let mut c = false;
        c |= ui.radio_value(&mut f.bots, Bots::Ninguno, "Sin bots").changed();
        c |= ui.radio_value(&mut f.bots, Bots::Fill, "Mantener N jugadores en total (humanos + bots)").changed();
        c |= ui.radio_value(&mut f.bots, Bots::Add, "Agregar N bots fijos").changed();
        let activo = f.bots != Bots::Ninguno;
        ui.add_enabled_ui(activo, |ui| {
            rejilla("bots_cantidad").show(ui, |ui| {
                c |= fila_texto(ui, "N", &mut f.bots_n);
                c |= fila_combo(ui, "Modo de relleno", &mut f.fillmode, FILL_MODOS);
            });
            c |= ui.checkbox(&mut f.kick, "Sacar bots cuando entran humanos").changed();
            c |= ui.checkbox(&mut f.spec, "Contar a quien este en espectador").changed();
            c |= ui.checkbox(&mut f.watch, "Solo hay bots mientras haya humanos").changed();
        });
        c
    })
}

fn bots_dificultad(ui: &mut egui::Ui, f: &mut Form) -> bool {
    tarjeta(ui, "Dificultad y equipos", "", |ui| {
        let mut c = false;
        rejilla("bots_dificultad").show(ui, |ui| {
            c |= fila_combo(ui, "Dificultad (0 aleatoria, 1 facil ... 7 dificil, 9 totalmente aleatoria)", &mut f.skill, SKILLS);
            c |= fila_texto(ui, "Dificultad minima (con 0)", &mut f.skmin);
            c |= fila_texto(ui, "Dificultad maxima (con 0)", &mut f.skmax);
            c |= fila_combo(ui, "Equipo de los bots", &mut f.equipo, EQUIPOS);
            c |= fila_texto(ui, "Bots forzados al equipo axis", &mut f.team_n);
            c |= fila_combo(ui, "Al repartir equipos cuenta solo bots (0/1)", &mut f.team_mode, &["0", "1"]);
        });
        c
    })
}

fn fila_texto(ui: &mut egui::Ui, etiqueta: &str, valor: &mut String) -> bool {
    ui.label(etiqueta);
    let c = ui.add(egui::TextEdit::singleline(valor).desired_width(CAMPO_W)).changed();
    ui.end_row();
    c
}

/// Desplegable de opciones; el valor guardado es la primera palabra de la opcion elegida.
fn fila_combo(ui: &mut egui::Ui, etiqueta: &str, valor: &mut String, opciones: &[&str]) -> bool {
    ui.label(etiqueta);
    let antes = valor.clone();
    let sel = opciones.iter().copied().find(|o| token(o) == valor.as_str()).unwrap_or(valor.as_str()).to_string();
    egui::ComboBox::from_id_salt(etiqueta).width(CAMPO_W).selected_text(sel).show_ui(ui, |ui| {
        for o in opciones {
            ui.selectable_value(valor, token(o).to_string(), *o);
        }
    });
    ui.end_row();
    *valor != antes
}

fn ruta_fila(ui: &mut egui::Ui, etiqueta: &str, valor: &mut String, carpeta: bool) {
    ui.label(etiqueta);
    ui.horizontal(|ui| {
        let ancho = ui.available_width() - 44.0;
        ui.add(egui::TextEdit::singleline(valor).desired_width(ancho));
        if ui.button("...").clicked() {
            let ruta = if carpeta { rfd::FileDialog::new().pick_folder() } else { rfd::FileDialog::new().pick_file() };
            if let Some(p) = ruta {
                *valor = p.to_string_lossy().into_owned();
            }
        }
    });
}
