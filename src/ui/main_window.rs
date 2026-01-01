use gtk4::{prelude::*, ApplicationWindow, HeaderBar, Button, Box, Image};
use std::sync::Arc;
use tokio::sync::Mutex;

pub struct MainWindow {
    pub window: ApplicationWindow,
    pub tray_icon: crate::ui::tray_icon::SystemTray,
    pub should_minimize_to_tray: bool,
    pub is_minimized_to_tray: bool,
    pub conflict_service: Option<Arc<tokio::sync::Mutex<crate::ui::conflict_dialog::ConflictDialogService>>>,
}

impl MainWindow {
    pub fn new(
        app: &gtk::Application, 
        start_minimized: bool,
        config_manager: Arc<Mutex<crate::core::config::ConfigManager>>
    ) -> Self {
        let window = ApplicationWindow::new(app);
        window.set_title(Some("CopyMaster"));
        window.set_default_size(1000, 700);
        
        // Cargar icono de ventana
        window.set_icon_name(Some("copymaster"));
        
        // Crear bandeja del sistema
        let tray_icon = crate::ui::tray_icon::SystemTray::new(app);
        
        // Configurar ventana para minimizar a la bandeja
        window.connect_close_request({
            let tray_icon = tray_icon.clone();
            move |window| {
                // Ocultar ventana en lugar de cerrarla
                window.hide();
                
                // Actualizar tooltip del icono
                tray_icon.set_tooltip("CopyMaster - Minimizado a la bandeja");
                
                // Prevenir el cierre real
                gtk::Inhibit(true)
            }
        });
        
        let instance = Self {
            window,
            tray_icon,
            should_minimize_to_tray: true,
            is_minimized_to_tray: false,
            conflict_service: None,
        };
        
        // Configurar menú de aplicación
        Self::setup_app_menu(app, config_manager.clone());
        
        // Si se solicita iniciar minimizado
        if start_minimized {
            instance.hide_to_tray();
        } else {
            instance.show_window();
        }
        
        instance
    }
    
    pub fn show_window(&self) {
        self.window.show();
        self.tray_icon.set_tooltip("CopyMaster - Activo");
        self.is_minimized_to_tray = false;
    }
    
    pub fn hide_to_tray(&self) {
        self.window.hide();
        self.tray_icon.set_tooltip("CopyMaster - En la bandeja");
        self.tray_icon.show();
        self.is_minimized_to_tray = true;
    }
    
    pub fn toggle_visibility(&self) {
        if self.window.is_visible() {
            self.hide_to_tray();
        } else {
            self.show_window();
        }
    }
    
    pub fn build_header_bar(&self) -> HeaderBar {
        let header = HeaderBar::new();
        
        // Botón para minimizar a la bandeja
        let minimize_btn = Button::from_icon_name("window-minimize-symbolic");
        minimize_btn.set_tooltip_text(Some("Minimizar a la bandeja"));
        
        minimize_btn.connect_clicked({
            let window = self.window.clone();
            let tray_icon = self.tray_icon.clone();
            move |_| {
                window.hide();
                tray_icon.set_tooltip("CopyMaster - Minimizado a la bandeja");
            }
        });
        
        // Botón para mostrar/ocultar desde la bandeja
        let tray_toggle_btn = Button::from_icon_name("view-restore-symbolic");
        tray_toggle_btn.set_tooltip_text(Some("Mostrar/Ocultar ventana"));
        
        tray_toggle_btn.connect_clicked({
            let window = self.window.clone();
            move |_| {
                if window.is_visible() {
                    window.hide();
                } else {
                    window.show();
                }
            }
        });
        
        header.pack_start(&minimize_btn);
        header.pack_end(&tray_toggle_btn);
        
        header
    }
    
    fn setup_app_menu(app: &gtk::Application, config_manager: Arc<Mutex<crate::core::config::ConfigManager>>) {
        let menu = gio::Menu::new();
        
        // Sección de archivo
        let file_section = gio::Menu::new();
        file_section.append(Some("Configuración"), Some("app.settings"));
        file_section.append(Some("Salir"), Some("app.quit"));
        menu.append_section(None, &file_section);
        
        // Sección de ayuda
        let help_section = gio::Menu::new();
        help_section.append(Some("Acerca de"), Some("app.about"));
        menu.append_section(None, &help_section);
        
        app.set_menubar(Some(&menu));
        
        // Conectar acción de configuración
        let settings_action = gio::SimpleAction::new("settings", None);
        settings_action.connect_activate({
            let config_manager = config_manager.clone();
            let app = app.clone();
            move |_, _| {
                let config_manager = config_manager.clone();
                let app = app.clone();
                
                tokio::spawn(async move {
                    if let Some(window) = app.active_window() {
                        let settings_dialog = crate::ui::settings_dialog::SettingsDialog::new(
                            Some(&window),
                            config_manager,
                        );
                        
                        let _ = settings_dialog.run().await;
                    }
                });
            }
        });
        
        app.add_action(&settings_action);
        
        // Conectar acción de salir
        let quit_action = gio::SimpleAction::new("quit", None);
        quit_action.connect_activate(move |_, _| {
            app.quit();
        });
        
        app.add_action(&quit_action);
        
        // Conectar acción acerca de
        let about_action = gio::SimpleAction::new("about", None);
        about_action.connect_activate({
            let app = app.clone();
            move |_, _| {
                let about = gtk4::AboutDialog::new();
                about.set_program_name("CopyMaster");
                about.set_version(Some("0.1.0"));
                about.set_comments(Some("Administrador de copias avanzado para Linux"));
                about.set_website(Some("https://github.com/tuusuario/copymaster"));
                about.set_logo_icon_name(Some("copymaster"));
                
                if let Some(window) = app.active_window() {
                    about.set_transient_for(Some(&window));
                }
                
                about.show();
            }
        });
        
        app.add_action(&about_action);
    }
    
    pub fn set_conflict_service(&mut self, service: Arc<tokio::sync::Mutex<crate::ui::conflict_dialog::ConflictDialogService>>) {
        self.conflict_service = Some(service);
    }
}