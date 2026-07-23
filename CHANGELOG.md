# Changelog
All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]
### Added
- implement `ping` command by invoking the system `ping` binary (cross-platform)
- implement `pin` command delegating to `ping` (async variant)
- add `GIT` constant for consistency with `NPM`
- extract `IP_DISCOVERY_HOST`/`IP_DISCOVERY_BIND` named constants from `get_agent_ip`
- add unit tests for `set_config` (invalid tool, git exit code 5 no-op)
- add unit tests for CLI argument parsing and dispatch
- add unit tests for `ping`/`pin` success and failure paths

### Changed
- separate `execute()` parsing from dispatch via `run()` returning `Option<String>`
- update README: `pin` description no longer references tokio specifically
- `enable_proxy` now rolls back the git proxy config if the npm config
  step fails, preventing a partially-configured state
- `disable_proxy` now restores the previous git proxy value if the npm
  config step fails during disable
- `disable_proxy` no longer fails when the proxy is not currently set (git
  returns exit code 5 when unsetting a non-existent key; this is now treated
  as a no-op instead of an error)
- `ping` and `pin` commands no longer silently ignore all CLI arguments
  (destination, count, size, ttl, interval)
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
