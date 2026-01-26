# Changelog

All notable changes to this project will be documented here.

The format is based on **Keep a Changelog**,
and this project follows **semantic-ish versioning**.

---

## [0.1.0] – Initial Release

### Added

- Core directory swapping algorithm (largest-first, deadlock-aware)
- Feasibility check before execution
- Dry-run mode with detailed output
- Streaming file moves (low RAM usage)
- Recursive directory scanning
- Reserve space protection
- CLI interface (`swapdirs`)
- Windows release support
- Nix / NixOS development environment

### Notes

- No journaling yet (abort is safe but not resumable)
- No GUI
- Designed for correctness over speed

---

## [Unreleased]

### Planned

- Journaling / resume
- Progress bars
- Same-disk rename optimization
- Improved error recovery
