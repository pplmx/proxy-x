# Changelog
All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [0.2.2] - 2026-07-24
### Added
- add crate-level (`//!`) documentation to the library root: an overview, a
  command table, how the proxy is applied to git and npm, and a usage example
- integration tests now verify all three proxy keys (git `http.proxy`, npm
  `proxy`, npm `https-proxy`) are set on enable and cleared on disable, and
  exercise the `status` binary end-to-end (disabled state and enabled URL)
- add doc examples across the public API, with executable doc-tests for the
  pure functions (`validate_proxy_url`, `ProxyStatus::is_disabled`) and
  `no_run` examples for the config/network-mutating ones
- add `status` command and a public `proxy_status()` API that report the
  current git/npm proxy configuration (read-only; a missing or erroring tool
  is reported as "not set" rather than failing)
- implement `ping` command by invoking the system `ping` binary (cross-platform)
- implement `pin` command delegating to `ping` (async variant)
- add `GIT` constant for consistency with `NPM`
- extract `IP_DISCOVERY_HOST`/`IP_DISCOVERY_BIND` named constants from `get_agent_ip`
- add unit tests for `set_config` (invalid tool, git exit code 5 no-op)
- add unit tests for CLI argument parsing and dispatch
- add unit tests for `ping`/`pin` success and failure paths

### Changed
- expand the README: promote Usage to a top-level section and document exactly
  what `enable`/`disable` configure (git `http.proxy`, which covers both http
  and https; npm `proxy` + `https-proxy`) plus the validate + rollback behavior
- `enable_proxy`/`disable_proxy` and the git/npm config readers now delegate to
  binary-injectable `*_with` helpers (public API unchanged). This makes the
  npm-failure rollback path unit-testable; added tests verifying git `http.proxy`
  is rolled back to its prior value (or unset when there was none) when npm
  fails during both enable and disable
- separate `execute()` parsing from dispatch via `run()` returning `Option<String>`
- update README: `pin` description no longer references tokio specifically
- add per-reply timeout (5s Linux/macOS, 5s Windows) to prevent hanging
  on unreachable hosts
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

### Fixed
- `ping` no longer re-wraps the spawn error from a missing `ping` binary, so
  the original `io::ErrorKind` (e.g. `NotFound`) is preserved for callers that
  match on it instead of being flattened to `ErrorKind::Other`. The command
  execution was extracted into an internal `execute_ping` helper so this path
  is unit-testable
- `enable`/`disable` now manage npm's `https-proxy` in addition to `proxy`.
  The default npm registry is `https://`, and npm uses `https-proxy` (not
  `proxy`) for https registries, so previously `npm install` did NOT route
  through the configured proxy. `enable` now sets both keys (with a full
  rollback of git `http.proxy`, npm `proxy`, and npm `https-proxy` on
  failure), `disable` clears both, and `status` reports `https-proxy`. git is
  unchanged: `http.proxy` already covers both http and https URLs
- `enable_proxy`'s rollback on an npm failure now restores the git proxy to
  its previous value instead of unconditionally unsetting it. Previously a
  partial failure silently wiped a proxy the user had configured before
  calling `enable`; the prior value is now preserved (and unset only when
  there was none). The previous-value lookup is shared with `disable_proxy`
  via a new internal `read_git_proxy` helper
- `ping`/`pin` `--count` (`-c`) is no longer capped at 255. The field was
  typed `u8`, so clap rejected any count above 255 even though the wrapped
  system `ping` accepts arbitrarily large counts; it is now `u64`, matching
  the `interval` field
- `enable_proxy` now validates the proxy URL (must include a scheme such as
  `http://`) before writing any config. Both `git` and `npm` silently accept
  scheme-less URLs like `localhost:8080`, which are then unusable as proxies;
  proxy-x now fails fast with a clear message instead
- `proxy-x` now exits with status 1 (non-zero) when a command fails, instead
  of printing an error to stderr and exiting 0. This lets shells, `&&`,
  `$?`, and CI detect failures

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
