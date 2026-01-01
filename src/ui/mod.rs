pub mod conflict_dialog;
pub mod settings_dialog;
pub mod drag_drop;
pub mod drop_dialog;
pub mod main_window;
pub mod queue_panel;
pub mod style;
pub mod tray_icon;
pub mod devices_panel;

// Re-export para facilitar el acceso
pub use conflict_dialog::ConflictDialogService;
pub use settings_dialog::SettingsDialog;
pub use main_window::MainWindow;