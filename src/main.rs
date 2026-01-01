mod cli;
mod core;
mod ui;

use cli::{parse_args, handle_cli_commands};
use gtk4::prelude::*;
use std::sync::Arc;
use tokio::sync::Mutex;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Parsear argumentos de línea de comandos
    let cli = parse_args();
    
    // Manejar comandos específicos
    handle_cli_commands(&cli).await?;
    
    // Cargar configuración
    let config_manager = Arc::new(Mutex::new(core::config::ConfigManager::new()?));
    let config = config_manager.lock().await.get_config().clone();
    
    // Inicializar GTK
    gtk4::init()?;
    
    let app = gtk4::Application::new(
        Some("org.copymaster"),
        gio::ApplicationFlags::empty(),
    );
    
    app.connect_startup(|_| {
        // Cargar recursos, estilos, etc.
        ui::style::load_css();
    });
    
    app.connect_activate(move |app| {
        // Crear ventana principal
        let main_window = ui::main_window::MainWindow::new(
            app, 
            cli.minimized || config.start_minimized,
            config_manager.clone()
        );
        
        // Crear servicio de diálogos de conflicto
        let (conflict_service, conflict_sender) = ui::conflict_dialog::ConflictDialogService::new(
            Some(main_window.window.clone()),
            config_manager.clone()
        );
        
        // Iniciar servicio de conflictos
        let conflict_service_arc = Arc::new(Mutex::new(conflict_service));
        tokio::spawn({
            let conflict_service = conflict_service_arc.clone();
            async move {
                let service = conflict_service.lock().await;
                // Nota: En realidad necesitaríamos consumir el service, 
                // así que esto es simplificado
            }
        });
        
        // Configurar auto-arranque si está habilitado
        if config.autostart_enabled && !cli.minimized {
            if config.start_minimized {
                main_window.hide_to_tray();
            }
        }
        
        // Si se ejecuta como daemon, solo mostrar icono en la bandeja
        if cli.daemon {
            main_window.hide_to_tray();
            
            let mut daemon = core::daemon::CopyMasterDaemon::new();
            tokio::spawn(async move {
                let _ = daemon.run_as_daemon().await;
            });
        }
        
        setup_signal_handlers(app, main_window);
    });
    
    // Ejecutar aplicación
    let args: Vec<String> = std::env::args().collect();
    app.run_with_args(&args);
    
    Ok(())
}

fn setup_signal_handlers(app: &gtk4::Application, window: ui::main_window::MainWindow) {
    // Manejar Ctrl+C en terminal
    ctrlc::set_handler(move || {
        println!("Recibida señal de interrupción. Cerrando...");
        app.quit();
    }).expect("Error configurando manejador de Ctrl+C");
    
    // Configurar para que el icono de la bandeja sobreviva al cierre de la ventana
    app.connect_shutdown(move |_| {
        println!("Cerrando CopyMaster...");
        // Guardar configuración, estado, etc.
    });
}