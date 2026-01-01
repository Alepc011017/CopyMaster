// src/core/optimizer.rs
use crate::core::device_detector::{DeviceInfo, DeviceType};
use crate::core::copy_engine::CopyOptions;

#[derive(Debug, Clone)]
pub struct CopyStrategy {
    pub buffer_size: usize,
    pub max_threads: usize,
    pub use_direct_io: bool,
    pub read_ahead: usize,
    pub throttle_mbps: Option<f64>,
}

impl Default for CopyStrategy {
    fn default() -> Self {
        Self {
            buffer_size: 65536,
            max_threads: num_cpus::get() * 2,
            use_direct_io: false,
            read_ahead: 4,
            throttle_mbps: None,
        }
    }
}

pub struct CopyOptimizer;

impl CopyOptimizer {
    pub fn new() -> Self {
        Self
    }
    
    pub fn get_optimal_strategy(
        &self,
        source: &DeviceInfo,
        dest: &DeviceInfo,
    ) -> CopyStrategy {
        match (&source.device_type, &dest.device_type) {
            (DeviceType::USB3, DeviceType::NVMeSSD) => {
                CopyStrategy {
                    buffer_size: 256 * 1024,
                    max_threads: 2,
                    use_direct_io: false,
                    read_ahead: 4,
                    ..Default::default()
                }
            }
            (DeviceType::NVMeSSD, DeviceType::USB3) => {
                CopyStrategy {
                    buffer_size: 128 * 1024,
                    max_threads: 4,
                    use_direct_io: false,
                    read_ahead: 2,
                    throttle_mbps: Some(400.0),
                }
            }
            (DeviceType::HDD, DeviceType::SataSSD) => {
                CopyStrategy {
                    buffer_size: 64 * 1024,
                    max_threads: 4,
                    use_direct_io: true,
                    read_ahead: 8,
                    ..Default::default()
                }
            }
            _ => CopyStrategy::default(),
        }
    }
    
    pub fn optimize_parameters(
        &self,
        source: &DeviceInfo,
        dest: &DeviceInfo,
        base_options: &CopyOptions
    ) -> CopyOptions {
        let mut optimized = base_options.clone();
        
        let slowest_speed = source.estimated_speed_mbps
            .and_then(|s1| dest.estimated_speed_mbps.map(|s2| s1.min(s2)))
            .unwrap_or(100.0);
        
        optimized.buffer_size = match slowest_speed {
            s if s < 50.0 => 16 * 1024,
            s if s < 200.0 => 32 * 1024,
            s if s < 1000.0 => 64 * 1024,
            _ => 128 * 1024,
        };
        
        match (&source.device_type, &dest.device_type) {
            (DeviceType::NVMeSSD, DeviceType::NVMeSSD) => {
                optimized.max_threads = num_cpus::get() * 4;
            }
            (_, DeviceType::USB2) | (DeviceType::USB2, _) => {
                optimized.max_threads = 2;
                optimized.buffer_size = 32 * 1024;
            }
            (DeviceType::HDD, _) | (_, DeviceType::HDD) => {
                optimized.max_threads = std::cmp::max(2, num_cpus::get() / 2);
            }
            _ => {}
        }
        
        optimized
    }
}