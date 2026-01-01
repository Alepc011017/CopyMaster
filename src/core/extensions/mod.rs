// src/core/extensions/mod.rs
// Directorio para extensiones comunitarias
// Las extensiones se pueden cargar dinámicamente

use std::collections::HashMap;
use std::sync::Arc;

pub mod community;

/// Registro de extensiones disponibles
pub struct ExtensionRegistry {
    extensions: HashMap<String, Box<dyn CopyExtension>>,
}

pub trait CopyExtension: Send + Sync {
    fn name(&self) -> &str;
    fn description(&self) -> &str;
    fn author(&self) -> &str;
    fn version(&self) -> &str;
    fn supports_feature(&self, feature: &str) -> bool;
    fn create_handler(&self) -> Box<dyn CopyHandler>;
}

pub trait CopyHandler: Send + Sync {
    fn execute(&self) -> Result<(), String>;
}

impl ExtensionRegistry {
    pub fn new() -> Self {
        Self {
            extensions: HashMap::new(),
        }
    }
    
    pub fn register(&mut self, name: String, extension: Box<dyn CopyExtension>) {
        self.extensions.insert(name, extension);
    }
    
    pub fn get(&self, name: &str) -> Option<&Box<dyn CopyExtension>> {
        self.extensions.get(name)
    }
    
    pub fn list(&self) -> Vec<String> {
        self.extensions.keys().cloned().collect()
    }
}