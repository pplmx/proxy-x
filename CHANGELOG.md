# Changelog
All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]
### Added
- `pin` command documentation in README
- `disable` command help description

### Changed
- move husky-rs to dev-dependencies (it's a dev tool, not needed at build time)
- remove local build.rs (husky-rs build script handles hook installation)
- update clap to ~4.6.0
- update husky-rs to 0.3
- set_config returns Result instead of panicking on invalid tool
- enable_proxy/disable_proxy return io::Result
- ping() / async_ping() return io::Result with proper error messages
- update the docs

## [0.2.1] - 2023-12-26
### Changed
- update version to `0.2.1` in `cargo.toml`

## [0.2.0] - 2023-12-26
### Added
- partially support command `ping`
- support command `ip`
- add commitizen to check if the commit message follows the [conventional commit specification](https://www.conventionalcommits.org/)
- add pre-commit hook

### Changed
- use clap `derive` instead of clap `builder`

## [0.1.1] - 2023-12-12
### Added
- support command `enable` and `disable`
