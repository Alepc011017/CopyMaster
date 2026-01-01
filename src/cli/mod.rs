use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(name = "copymaster")]
#[command(author = "CopyMaster Team")]
#[command(version = "0.1.0")]
#[command(about = "Administrador de copias avanzado para Linux", long_about = None)]
pub struct Cli {
    /// Iniciar minimizado en la bandeja del sistema
    #[arg(short, long)]
    pub minimized: bool,
    
    /// No mostrar interfaz gráfica (modo daemon)
    #[arg(short = 'D', long)]
    pub daemon: bool,
    
    /// Comandos de configuración
    #[command(subcommand)]
    pub command: Option<Commands>,
}

#[derive(Subcommand)]
pub enum Commands {
    /// Configurar auto-arranque
    Autostart {
        /// Habilitar auto-arranque
        #[arg(short, long)]
        enable: bool,
        
        /// Deshabilitar auto-arranque
        #[arg(short, long)]
        disable: bool,
        
        /// Iniciar minimizado
        #[arg(short, long)]
        minimized: bool,
    },
    
    /// Instalar en el sistema
    Install {
        /// Instalar iconos y archivos .desktop
        #[arg(short, long)]
        with_desktop: bool,
    },
    
    /// Mostrar información del sistema
    SystemInfo,
}

pub fn parse_args() -> Cli {
    Cli::parse()
}

pub async fn handle_cli_commands(cli: &Cli) -> Result<(), Box<dyn std::error::Error>> {
    if let Some(command) = &cli.command {
        match command {
            Commands::Autostart { enable, disable, minimized } => {
                let autostart = crate::core::autostart::AutoStartManager::new()?;
                
                if *enable {
                    autostart.enable(*minimized)?;
                    println!("✓ Auto-arranque habilitado");
                } else if *disable {
                    autostart.disable()?;
                    println!("✓ Auto-arranque deshabilitado");
                } else {
                    if autostart.is_enabled() {
                        println!("Auto-arranque está HABILITADO");
                    } else {
                        println!("Auto-arranque está DESHABILITADO");
                    }
                }
                
                // Salir después de procesar comando de autostart
                std::process::exit(0);
            }
            Commands::Install { with_desktop } => {
                if *with_desktop {
                    // Instalar entrada en el menú de aplicaciones
                    let autostart = crate::core::autostart::AutoStartManager::new()?;
                    autostart.create_app_menu_entry()?;
                    println!("✓ Entrada de menú creada");
                }
                println!("✓ Instalación completada");
                std::process::exit(0);
            }
            Commands::SystemInfo => {
                println!("CopyMaster System Info:");
                println!("  Versión: 0.1.0");
                println!("  Entorno de escritorio: {:?}", 
                    crate::core::autostart::DesktopEnvironment::detect());
                
                let autostart = crate::core::autostart::AutoStartManager::new()?;
                println!("  Auto-arranque: {}", 
                    if autostart.is_enabled() { "HABILITADO" } else { "DESHABILITADO" });
                
                std::process::exit(0);
            }
        }
    }
    
    Ok(())
}