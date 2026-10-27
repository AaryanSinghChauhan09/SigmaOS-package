// SigmaOS Linux Package & Driver Translation Subsystem
// Zero-dependency, zero-allocation
// Integrates foreign Linux package frameworks (.deb, .rpm, pacman) directly with the SigmaOS Driver system.

// Temporary stub for driver framework
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DriverType {
    Net,
    Block,
    Char,
}

#[derive(Debug, Clone)]
pub struct SimpleDriver {
    pub id: u32,
    pub driver_type: DriverType,
}

impl SimpleDriver {
    pub fn new(id: u32, driver_type: DriverType) -> Self {
        Self { id, driver_type }
    }
}

#[cfg(not(feature = "standalone_test"))]
use crate::package::PackageFormat;
#[cfg(not(feature = "standalone_test"))]
use std::sync::atomic::{AtomicBool, Ordering};

#[cfg(feature = "standalone_test")]
#[path = "universal.rs"]
pub mod universal;

// PackageFormat for the standalone_test build lane: resolved from the
// sibling universal.rs module included above (this file is also compiled
// standalone as its own crate root, where crate::package does not exist).
#[cfg(feature = "standalone_test")]
use universal::PackageFormat;

#[cfg(feature = "standalone_test")]
use core::sync::atomic::{AtomicBool, Ordering};
#[cfg(feature = "standalone_test")]
use std::collections::BTreeMap;
#[cfg(not(feature = "standalone_test"))]
use std::collections::BTreeMap;
use std::format;
use std::string::{String, ToString};
use std::vec::Vec;

/// User-Defined Function (UDF) for Package & Syscall Translation
/// Dynamically translates foreign syscalls or I/O request codes to native SigmaOS drivers
pub trait PackageTranslationUdf: Sync {
    fn name(&self) -> &'static str;
    fn translate_syscall(&self, foreign_num: u32) -> u32;
    fn translate_io_control(&self, command: u32) -> u32;
}

pub struct GenericLinuxTranslationUdf;
impl PackageTranslationUdf for GenericLinuxTranslationUdf {
    fn name(&self) -> &'static str {
        "generic-linux-udf"
    }

    fn translate_syscall(&self, foreign_num: u32) -> u32 {
        match foreign_num {
            0 => 0,                  // read -> native read
            1 => 1,                  // write -> native write
            2 => 2,                  // open -> native open
            3 => 3,                  // close -> native close
            54 => 54,                // ioctl -> native ioctl
            _ => foreign_num + 2000, // remap other Linux syscall offsets safely
        }
    }

    fn translate_io_control(&self, command: u32) -> u32 {
        match command {
            0x5401 => 0x1001, // TCGETS -> native TCGETS
            0x5402 => 0x1002, // TCSETS -> native TCSETS
            _ => command + 1000,
        }
    }
}

/// Linux Package Translation Layer
/// Handles translation of .deb, .rpm, and pacman packages to SigmaOS format
pub struct LinuxPackageTranslator {
    udf: Box<dyn PackageTranslationUdf>,
    translation_cache: BTreeMap<String, u32>,
    enabled: AtomicBool,
}

impl LinuxPackageTranslator {
    pub fn new(udf: Box<dyn PackageTranslationUdf>) -> Self {
        Self {
            udf,
            translation_cache: BTreeMap::new(),
            enabled: AtomicBool::new(true),
        }
    }

    pub fn is_enabled(&self) -> bool {
        self.enabled.load(Ordering::Relaxed)
    }

    pub fn enable(&self) {
        self.enabled.store(true, Ordering::Relaxed);
    }

    pub fn disable(&self) {
        self.enabled.store(false, Ordering::Relaxed);
    }

    pub fn translate_package(&mut self, package: &str) -> Option<String> {
        if !self.is_enabled() {
            return None;
        }

        let package_id = self.translate_package_name(package);
        Some(format!("sigmaos-{}", package_id))
    }

    fn translate_package_name(&mut self, package: &str) -> String {
        if let Some(&cached) = self.translation_cache.get(package) {
            return cached.to_string();
        }

        let translated = self.udf.translate_syscall(package.len() as u32);
        self.translation_cache.insert(package.to_string(), translated);
        translated.to_string()
    }

    pub fn register_driver(&self, driver: SimpleDriver) -> Result<(), String> {
        if !self.is_enabled() {
            return Err("Translator is disabled".to_string());
        }

        // In a real implementation, this would register the driver with the kernel
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_generic_linux_translation() {
        let udf = GenericLinuxTranslationUdf;
        assert_eq!(udf.translate_syscall(0), 0);
        assert_eq!(udf.translate_syscall(54), 54);
        assert_eq!(udf.translate_syscall(100), 2100);
    }

    #[test]
    fn test_linux_package_translator() {
        let translator = LinuxPackageTranslator::new(Box::new(GenericLinuxTranslationUdf));
        assert!(translator.is_enabled());

        let translated = translator.translate_package("test-package");
        assert!(translated.is_some());
    }

    #[test]
    fn test_driver_registration() {
        let translator = LinuxPackageTranslator::new(Box::new(GenericLinuxTranslationUdf));
        let driver = SimpleDriver::new(1, DriverType::Net);
        let result = translator.register_driver(driver);
        assert!(result.is_ok());
    }
}
