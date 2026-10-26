//! SigmaOS Package Manager
//! Universal package management with multiple backend support

extern crate sigmaos_klib as klib;

pub mod package;

// Temporarily disable complex modules to fix compilation
// pub mod sigpkg;
// pub mod pacman;
// pub mod package_mgr;

pub use package::PackageManager;
pub use package::PackageBackend;

// Re-export klib types for use in package
pub use crate::klib::{HashMap, BTreeMap, HashSet, Vec};
