// SPDX-License-Identifier: Apache-2.0
//! SigmaOS-package — SigmaOS package component inspired by Linux Mint / Omarchy / Arch Linux.
//!
//! Arch Linux PKGBUILD model: validated metadata, source+checksum pairs, deterministic render.
//! Principles: single responsibility, fail-fast, defensive programming,
//! component isolation, stable API, deterministic tests.

#![forbid(unsafe_code)]

use std::fmt;
use std::collections::BTreeMap;

/// Errors for `PkgbuildError` operations. Fail-fast: invalid input is rejected.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PkgbuildError {
    /// Required PKGBUILD field must not be empty
    EmptyField(String),
    /// Package name violates pacman naming rules
    BadName(String),
    /// pkgrel must be a positive integer
    BadPkgrel(String),
    /// Unsupported target architecture
    BadArch(String),
    /// Source URL already listed
    DuplicateSource(String),
    /// sha256 array length differs from source count
    ChecksumMismatch,
}

impl fmt::Display for PkgbuildError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::EmptyField(x) => write!(f, "required pkgbuild field must not be empty: {x}"),
            Self::BadName(x) => write!(f, "package name violates pacman naming rules: {x}"),
            Self::BadPkgrel(x) => write!(f, "pkgrel must be a positive integer: {x}"),
            Self::BadArch(x) => write!(f, "unsupported target architecture: {x}"),
            Self::DuplicateSource(x) => write!(f, "source url already listed: {x}"),
            Self::ChecksumMismatch => write!(f, "sha256 array length differs from source count"),
        }
    }
}

impl std::error::Error for PkgbuildError {}

/// Arch-supported target architectures (subset of `pacman`'s arch array).
pub const ARCHES: &[&str] = &["x86_64", "any", "aarch64", "armv7h", "i686"];

/// A PKGBUILD package definition (Arch Linux packaging metadata).
#[derive(Debug, Default)]
pub struct Pkgbuild {
    name: String,
    pkgver: String,
    pkgrel: u32,
    arch: Vec<String>,
    depends: Vec<String>,
    makedepends: Vec<String>,
    sources: Vec<(String, String)>,
    env: BTreeMap<String, String>,
}

impl Pkgbuild {
    /// Empty package definition.
    pub fn new() -> Self { Self::default() }

    /// Set package name; validated against pacman rules `[a-z0-9@._+-]`.
    pub fn name(mut self, v: impl Into<String>) -> Result<Self, PkgbuildError> {
        let v = v.into();
        let ok = !v.is_empty()
            && v.chars().all(|c| c.is_ascii_lowercase() || c.is_ascii_digit()
                || matches!(c, '@' | '.' | '_' | '+' | '-'));
        if !ok { return Err(PkgbuildError::BadName(v)); }
        self.name = v;
        Ok(self)
    }

    /// Set upstream version; must not be empty or contain '-'.
    pub fn pkgver(mut self, v: impl Into<String>) -> Result<Self, PkgbuildError> {
        let v = v.into();
        if v.is_empty() || v.contains('-') { return Err(PkgbuildError::EmptyField("pkgver".into())); }
        self.pkgver = v;
        Ok(self)
    }

    /// Set package release; must be >= 1.
    pub fn pkgrel(mut self, v: u32) -> Result<Self, PkgbuildError> {
        if v == 0 { return Err(PkgbuildError::BadPkgrel(v.to_string())); }
        self.pkgrel = v;
        Ok(self)
    }

    /// Add a target architecture; must be in [`ARCHES`].
    pub fn arch(mut self, v: impl Into<String>) -> Result<Self, PkgbuildError> {
        let v = v.into();
        if !ARCHES.contains(&v.as_str()) { return Err(PkgbuildError::BadArch(v)); }
        if !self.arch.contains(&v) { self.arch.push(v); }
        Ok(self)
    }

    /// Add a runtime dependency.
    pub fn depends(mut self, v: impl Into<String>) -> Self {
        self.depends.push(v.into());
        self
    }

    /// Add a build-time dependency.
    pub fn makedepends(mut self, v: impl Into<String>) -> Self {
        self.makedepends.push(v.into());
        self
    }

    /// Add a `source = url` + `sha256sums` pair; rejects duplicate URLs.
    pub fn source(mut self, url: impl Into<String>, sha256: impl Into<String>)
        -> Result<Self, PkgbuildError>
    {
        let url = url.into();
        if self.sources.iter().any(|(u, _)| u == &url) {
            return Err(PkgbuildError::DuplicateSource(url));
        }
        self.sources.push((url, sha256.into()));
        Ok(self)
    }

    /// Set a PKGBUILD `prepare()`/environment variable.
    pub fn env(mut self, k: impl Into<String>, v: impl Into<String>) -> Self {
        self.env.insert(k.into(), v.into());
        self
    }

    /// Validate all required fields are present.
    pub fn validate(&self) -> Result<(), PkgbuildError> {
        for (field, val) in [("name", &self.name), ("pkgver", &self.pkgver)] {
            if val.is_empty() { return Err(PkgbuildError::EmptyField(field.into())); }
        }
        if self.pkgrel == 0 { return Err(PkgbuildError::BadPkgrel(self.pkgrel.to_string())); }
        if self.arch.is_empty() { return Err(PkgbuildError::EmptyField("arch".into())); }
        for a in &self.arch {
            if !ARCHES.contains(&a.as_str()) { return Err(PkgbuildError::BadArch(a.clone())); }
        }
        Ok(())
    }

    /// Render deterministic PKGBUILD text (sorted env, fixed field order).
    pub fn render(&self) -> Result<String, PkgbuildError> {
        self.validate()?;
        let mut out = String::new();
        out.push_str(&format!("# Maintainer: SigmaOS <sigmaos@localhost>\n"));
        out.push_str(&format!("pkgname={}\n", self.name));
        out.push_str(&format!("pkgver={}\n", self.pkgver));
        out.push_str(&format!("pkgrel={}\n", self.pkgrel));
        let arch = self.arch.join(" ");
        out.push_str(&format!("arch=({})\n", arch));
        if !self.depends.is_empty() {
            out.push_str(&format!("depends=({})\n",
                self.depends.iter().map(|d| format!("\"{}\"", d)).collect::<Vec<_>>().join(" ")));
        }
        if !self.makedepends.is_empty() {
            out.push_str(&format!("makedepends=({})\n",
                self.makedepends.iter().map(|d| format!("\"{}\"", d)).collect::<Vec<_>>().join(" ")));
        }
        let urls = self.sources.iter().map(|(u, _)| format!("\"{}\"", u)).collect::<Vec<_>>().join(" ");
        let sums = self.sources.iter().map(|(_, s)| format!("\"{}\"", s)).collect::<Vec<_>>().join(" ");
        out.push_str(&format!("source=({})\n", urls));
        out.push_str(&format!("sha256sums=({})\n", sums));
        for (k, v) in &self.env {
            out.push_str(&format!("export {k}=\"{v}\"\n"));
        }
        Ok(out)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn build() -> Result<Pkgbuild, PkgbuildError> {
        Ok(Pkgbuild::new()
            .name("hello-world")?
            .pkgver("1.0.0")?
            .pkgrel(1)?
            .arch("x86_64")?
            .arch("any")?
            .depends("bash")
            .source("https://example.com/a.tar.gz", "deadbeef")?
            .env("CFLAGS", "-O2"))
    }

    #[test]
    fn renders_valid_pkgbuild() {
        let text = build().unwrap().render().unwrap();
        assert!(text.contains("pkgname=hello-world"));
        assert!(text.contains("pkgver=1.0.0"));
        assert!(text.contains("arch=(x86_64 any)"));
        assert!(text.contains("sha256sums=(\"deadbeef\")"));
        assert!(text.contains("export CFLAGS=\"-O2\""));
    }

    #[test]
    fn bad_name_fails() {
        assert!(matches!(Pkgbuild::new().name("Bad Name!"), Err(PkgbuildError::BadName(_))));
        assert!(matches!(Pkgbuild::new().name("ok-name_1"), Ok(_)));
    }

    #[test]
    fn pkgrel_zero_fails() {
        assert!(matches!(Pkgbuild::new().pkgrel(0), Err(PkgbuildError::BadPkgrel(_))));
    }

    #[test]
    fn unknown_arch_fails() {
        assert!(matches!(Pkgbuild::new().arch("mips"), Err(PkgbuildError::BadArch(_))));
    }

    #[test]
    fn duplicate_source_fails() {
        let mut p = build().unwrap();
        assert!(matches!(p.source("https://example.com/a.tar.gz", "x"), Err(PkgbuildError::DuplicateSource(_))));
    }

    #[test]
    fn render_requires_required_fields() {
        assert!(matches!(Pkgbuild::new().render(), Err(PkgbuildError::EmptyField(_))));
    }

}

/// Backwards-compatible component handle (kept for existing callers/tests).
#[derive(Debug, Default)]
pub struct Component {
    name: &'static str,
}

impl Component {
    /// Create the component handle.
    pub fn new() -> Self {
        Self { name: "SigmaOS Component" }
    }
}
