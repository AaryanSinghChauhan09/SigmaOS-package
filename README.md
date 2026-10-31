# SigmaOS Package

Rust library for package and repository management experiments across several
distribution formats. The public module tree includes package resolution,
repository configuration, APT/pacman/Alpine/Gentoo integrations, signing,
sandboxing, update management, and delta metadata.

## Current implementation limits

The crate exposes a broad API surface, and `src/lib.rs` explicitly leaves some
modules disabled while compilation issues are addressed. Public types and
module names do not prove that an install path is complete, secure, or
compatible with a real package database. Do not run this library against a
production host or treat its verification APIs as established until the
relevant backend and threat model have been reviewed and exercised.

Package operations should be deterministic, privilege-aware, and auditable.
Before applying changes, a backend needs explicit signature and provenance
verification, transaction boundaries, failure recovery, and a safe preview of
the exact filesystem changes.

## Development

The crate has a sibling path dependency on `SigmaOS-klib`. The repository CI
workflow resolves SigmaOS path dependencies and runs `cargo test --all-targets
--locked`; its result must be checked for the commit under review before
claiming a passing build.
