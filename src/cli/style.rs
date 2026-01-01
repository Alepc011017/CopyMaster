// src/ui/style.rs
use gtk4::{gdk, gio, CssProvider, StyleContext};
use std::path::PathBuf;

pub fn load_css_and_icons() {
    // Cargar CSS
    let css_provider = CssProvider::new();
    
    // CSS por defecto con soporte para iconos
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
    ";
    
    css_provider.load_from_data(default_css);
    
    // Añadir provider al display
    StyleContext::add_provider_for_display(
        &gdk::Display::default().expect("No hay display"),
        &css_provider,
        gtk::STYLE_PROVIDER_PRIORITY_APPLICATION,
    );
    
    // Cargar CSS personalizado si existe
    if let Ok(mut css_path) = std::env::current_dir() {
        css_path.push("data");
        css_path.push("style.css");
        
        if css_path.exists() {
            if let Ok(file) = gio::File::for_path(&css_path) {
                let _ = css_provider.load_from_file(&file);
            }
        }
    }
    
    // Configurar icon theme
    setup_icon_theme();
}

fn setup_icon_theme() {
    // Añadir nuestro directorio de iconos al tema de iconos
    if let Ok(mut icon_dir) = std::env::current_dir() {
        icon_dir.push("data");
        icon_dir.push("icons");
        
        if icon_dir.exists() {
            let theme = gtk::IconTheme::for_display(
                &gdk::Display::default().expect("No hay display")
            );
            
            // Añadir nuestro directorio de iconos
            theme.add_search_path(&icon_dir.to_string_lossy());
            
            // También buscar en directorios estándar
            let standard_paths = vec![
                "/usr/share/icons",
                "/usr/local/share/icons",
                dirs::data_dir().map(|p| p.join("icons")).unwrap_or_default(),
            ];
            
            for path in standard_paths {
                if PathBuf::from(&path).exists() {
                    theme.add_search_path(&path.to_string_lossy());
                }
            }
        }
    }
}

/// Función para obtener icono por nombre y tamaño
pub fn get_icon_pixbuf(icon_name: &str, size: i32) -> Option<gdk_pixbuf::Pixbuf> {
    let theme = gtk::IconTheme::for_display(
        &gdk::Display::default().expect("No hay display")
    );
    
    // Primero intentar con nuestro icono personalizado
    let icon_names = vec![
        format!("copymaster-{}", icon_name),
        icon_name.to_string(),
        format!("system-{}", icon_name),
    ];
    
    for name in icon_names {
        if let Ok(pixbuf) = theme.load_icon(&name, size, gtk::IconLookupFlags::FORCE_SIZE) {
            return Some(pixbuf);
        }
    }
    
    // Si no se encuentra, usar icono por defecto
    theme.load_icon("image-missing", size, gtk::IconLookupFlags::FORCE_SIZE).ok()
}

/// Función para crear un Image widget con icono
pub fn create_icon_image(icon_name: &str, size: i32) -> gtk::Image {
    if let Some(pixbuf) = get_icon_pixbuf(icon_name, size) {
        gtk::Image::from_pixbuf(Some(&pixbuf))
    } else {
        // Icono de fallback
        gtk::Image::from_icon_name(icon_name)
    }
}