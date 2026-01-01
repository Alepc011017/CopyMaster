// src/ui/style.rs
use gtk4::{gdk, gio, CssProvider, StyleContext, IconTheme};
use std::path::PathBuf;

pub fn load_css() {
    let provider = CssProvider::new();
    
    // CSS por defecto
    let default_css = "
    /* Estilos para CopyMaster */
    window {
        background-color: @theme_bg_color;
    }
    
    .icon-button {
        min-width: 32px;
        min-height: 32px;
        padding: 4px;
    }
    
    .device-icon {
        min-width: 48px;
        min-height: 48px;
    }
    
    .copy-icon {
        color: @theme_selected_bg_color;
    }
    
    .pause-icon {
        color: @warning_color;
    }
    
    .stop-icon {
        color: @error_color;
    }
    
    .progress-icon {
        color: @theme_selected_bg_color;
    }
    
    /* Tray icon styles */
    .tray-icon-menu {
        padding: 8px;
    }
    
    .tray-icon-item {
        padding: 6px 12px;
    }
    
    .tray-icon-item:hover {
        background-color: alpha(@theme_fg_color, 0.1);
    }
    
    /* Barra de progreso */
    progressbar trough {
        min-height: 8px;
        border-radius: 4px;
    }
    
    progressbar progress {
        min-height: 8px;
        border-radius: 4px;
    }
    ";
    
    provider.load_from_data(default_css);
    
    // Añadir provider al display
    StyleContext::add_provider_for_display(
        &gdk::Display::default().expect("No hay display"),
        &provider,
        gtk::STYLE_PROVIDER_PRIORITY_APPLICATION,
    );
    
    // Cargar CSS personalizado si existe
    if let Ok(mut css_path) = std::env::current_dir() {
        css_path.push("data");
        css_path.push("style.css");
        
        if css_path.exists() {
            if let Ok(file) = gio::File::for_path(&css_path) {
                let _ = provider.load_from_file(&file);
            }
        }
    }
}

pub fn setup_icon_theme() {
    let theme = IconTheme::for_display(
        &gdk::Display::default().expect("No hay display")
    );
    
    // Añadir nuestro directorio de iconos si existe
    if let Ok(mut icon_dir) = std::env::current_dir() {
        icon_dir.push("data");
        icon_dir.push("icons");
        
        if icon_dir.exists() {
            theme.add_search_path(&icon_dir.to_string_lossy());
        }
    }
}