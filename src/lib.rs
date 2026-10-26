//! SigmaOS Package Manager
//! Universal package management with multiple backend support

pub mod package;
pub mod sigpkg;
pub mod pacman;
pub mod package_mgr;

pub use package::PackageManager;
pub use package::PackageBackend;
