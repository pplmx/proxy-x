//! # proxy-x
//!
//! A small cross-platform tool and library for managing HTTP(S) proxy settings
//! for `git` and `npm`, plus a few network diagnostics.
//!
//! ## Commands
//!
//! | Command | Effect |
//! |---------|--------|
//! | `enable <url>` | Set the proxy for git and npm |
//! | `disable` | Clear the proxy for git and npm (idempotent) |
//! | `status` | Show the current proxy configuration |
//! | `ip` | Print the local outbound IP address |
//! | `ping <host>` / `pin <host>` | ICMP ping via the system `ping` binary |
//!
//! ## How the proxy is applied
//!
//! - **git**: sets the global `http.proxy`. Despite the name, git uses this key
//!   for both `http://` and `https://` URLs, so a single value covers all git
//!   traffic.
//! - **npm**: sets both `proxy` (used for http registries) and `https-proxy`
//!   (used for https registries). The default registry is `https://`, so
//!   `https-proxy` is required for `npm install` to actually use the proxy.
//!
//! [`enable_proxy`] validates the URL (it must include a scheme such as
//! `http://`) and rolls back any partially-written config if a step fails, so
//! the system is never left half-configured. [`disable_proxy`] restores the
//! previous git proxy on a partial failure.
//!
//! ## Library usage
//!
//! ```no_run
//! proxy_x::enable_proxy("http://127.0.0.1:7890").unwrap();
//! let status = proxy_x::proxy_status();
//! println!("proxy disabled? {}", status.is_disabled());
//! proxy_x::disable_proxy().unwrap();
//! ```

use std::io;
use std::net::UdpSocket;
use std::process::Command;

pub mod pin;
pub mod ping;

/// The git binary name.
///
/// Using a constant avoids string-literal duplication and prevents
/// typos across `set_config` and `disable_proxy`.
pub const GIT: &str = "git";

#[cfg(windows)]
pub const NPM: &str = "npm.cmd";

#[cfg(not(windows))]
pub const NPM: &str = "npm";

/// Host:port used by [`get_agent_ip`] to determine the local IP.
///
/// Connecting a UDP socket to this address (without sending data) reveals
/// which local interface the kernel would use for outbound traffic, without
/// actually sending any packets or requiring network reachability.
const IP_DISCOVERY_HOST: &str = "8.8.8.8:53";

/// Bind address used by [`get_agent_ip`] to create an ephemeral UDP socket.
const IP_DISCOVERY_BIND: &str = "0.0.0.0:0";

/// Validate a proxy URL before storing it.
///
/// Both `git config` and `npm config set` accept scheme-less or otherwise
/// malformed proxy URLs at write time (git stores them silently; npm only
/// warns on read). Storing such a value produces a broken proxy that is hard
/// to debug, so we fail fast with a clear message instead.
///
/// A URL is considered valid when, after trimming surrounding whitespace, it
/// is non-empty and contains a scheme separator (`://`). This accepts every
/// common proxy scheme (`http`, `https`, `socks4`, `socks5`, `socks5h`,
/// `http://` with userinfo) while rejecting bare `host:port` values.
///
/// # Examples
///
/// ```
/// use proxy_x::validate_proxy_url;
///
/// assert!(validate_proxy_url("http://127.0.0.1:7890").is_ok());
/// assert!(validate_proxy_url("socks5h://user:pass@host:1080").is_ok());
/// // Bare host:port is rejected: git/npm store it silently but it is unusable.
/// assert!(validate_proxy_url("127.0.0.1:7890").is_err());
/// assert!(validate_proxy_url("").is_err());
/// ```
pub fn validate_proxy_url(url: &str) -> io::Result<()> {
    let trimmed = url.trim();
    if trimmed.is_empty() {
        return Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            "proxy URL must not be empty",
        ));
    }
    if !trimmed.contains("://") {
        return Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            format!(
                "proxy URL must include a scheme (e.g. \"http://\") \
                 because bare host:port values are silently stored by git \
                 and npm but are unusable as proxies; got: {trimmed}"
            ),
        ));
    }
    Ok(())
}

/// Enable the proxy for git and npm.
///
/// Sets git's global `http.proxy` and npm's `proxy` + `https-proxy` to
/// `proxy_url`. If the npm step fails, the git config is rolled back to its
/// prior value so the system is never left half-configured.
///
/// # Examples
///
/// ```no_run
/// proxy_x::enable_proxy("http://127.0.0.1:7890").expect("failed to enable proxy");
/// ```
pub fn enable_proxy(proxy_url: &str) -> io::Result<()> {
    enable_proxy_with(proxy_url, GIT, NPM)
}

/// Implementation of [`enable_proxy`] with injectable git/npm binaries.
///
/// Taking the binary names as parameters (instead of hardcoding [`GIT`]/[`NPM`])
/// lets tests point npm at a non-existent binary to exercise the rollback path
/// without needing npm to actually fail.
fn enable_proxy_with(proxy_url: &str, git: &str, npm: &str) -> io::Result<()> {
    // Validate up front so an invalid URL never leaves a half-configured
    // proxy behind if a later step fails.
    validate_proxy_url(proxy_url)?;

    // Capture the prior value of every key we touch so that a partial failure
    // can be rolled back to exactly where the user started, rather than
    // silently dropping a proxy they had configured before calling `enable`.
    let git_before = read_git_proxy_with(git);
    let npm_proxy_before = read_npm_config_with(npm, "proxy");
    let npm_https_before = read_npm_config_with(npm, "https-proxy");

    set_config("http.proxy", Some(proxy_url), git)?;

    // npm distinguishes `proxy` (used for http registries) from `https-proxy`
    // (used for https registries). The default registry is https, so setting
    // only `proxy` would NOT proxy `npm install`; both keys must be set.
    let npm_result =
        set_config("proxy", Some(proxy_url), npm).and_then(|()| set_config("https-proxy", Some(proxy_url), npm));
    if let Err(e) = npm_result {
        // Roll back all three keys to their previous values (unset when there
        // was none) to avoid leaving the system in a partial/inconsistent state.
        let _ = set_config("http.proxy", git_before.as_deref(), git);
        let _ = set_config("proxy", npm_proxy_before.as_deref(), npm);
        let _ = set_config("https-proxy", npm_https_before.as_deref(), npm);
        return Err(e);
    }
    println!("Proxy enabled");
    Ok(())
}

/// Disable the proxy for git and npm.
///
/// Unsets git's global `http.proxy` and npm's `proxy` + `https-proxy`.
/// Idempotent: succeeds even if no proxy is currently set. If the npm step
/// fails, the previous git proxy value is restored.
///
/// # Examples
///
/// ```no_run
/// proxy_x::disable_proxy().expect("failed to disable proxy");
/// ```
pub fn disable_proxy() -> io::Result<()> {
    disable_proxy_with(GIT, NPM)
}

/// Implementation of [`disable_proxy`] with injectable git/npm binaries.
fn disable_proxy_with(git: &str, npm: &str) -> io::Result<()> {
    // Save the current git proxy value so we can roll back if npm fails.
    let git_before = read_git_proxy_with(git);

    set_config("http.proxy", None, git)?;

    // Unset both npm proxy keys (see enable_proxy for why both are managed).
    let npm_result = set_config("proxy", None, npm).and_then(|()| set_config("https-proxy", None, npm));
    if let Err(e) = npm_result {
        // Rollback: git proxy was unset, but npm failed.
        // Attempt to restore the previous git proxy value (npm is best-effort).
        if let Some(prev) = git_before {
            let _ = set_config("http.proxy", Some(&prev), git);
        }
        return Err(e);
    }
    println!("Proxy disabled");
    Ok(())
}

/// Read the current global git `http.proxy` value (using the [`GIT`] binary).
///
/// Returns `None` when the key is unset, empty, or git is unavailable.
fn read_git_proxy() -> Option<String> {
    read_git_proxy_with(GIT)
}

/// Read the global git `http.proxy` value using the given git binary.
///
/// Shared by [`enable_proxy_with`] and [`disable_proxy_with`] so both can
/// restore the previous proxy on a partial (npm) failure instead of silently
/// dropping it. The binary is injectable for tests.
fn read_git_proxy_with(git: &str) -> Option<String> {
    Command::new(git)
        .args(["config", "--global", "http.proxy"])
        .output()
        .ok()
        .and_then(|o| {
            if o.status.success() {
                let value = String::from_utf8_lossy(&o.stdout).trim().to_string();
                (!value.is_empty()).then_some(value)
            } else {
                None
            }
        })
}

/// Read an npm config value by key using the [`NPM`] binary
/// (e.g. `"proxy"` or `"https-proxy"`).
///
/// Returns `None` when the key is unset (npm prints a sentinel), the output is
/// empty, or npm is unavailable.
fn read_npm_config(key: &str) -> Option<String> {
    read_npm_config_with(NPM, key)
}

/// Read an npm config value by key using the given npm binary.
///
/// Shared by [`enable_proxy_with`], [`disable_proxy_with`], and
/// [`proxy_status`]. The binary is injectable for tests.
fn read_npm_config_with(npm: &str, key: &str) -> Option<String> {
    let output = Command::new(npm).args(["config", "get", key]).output().ok()?;
    if !output.status.success() {
        return None;
    }
    parse_npm_config_output(&String::from_utf8_lossy(&output.stdout))
}

/// Parse the stdout of `npm config get <key>`.
///
/// npm prints the value (plus a trailing newline) when the key is set, or a
/// sentinel when it is not: newer npm versions print `null`, older ones print
/// `undefined`. Returns `None` for any unset/empty value.
fn parse_npm_config_output(stdout: &str) -> Option<String> {
    let value = stdout.trim();
    if value.is_empty() || value == "undefined" || value == "null" {
        None
    } else {
        Some(value.to_string())
    }
}

/// A read-only snapshot of the proxy configuration reported by git and npm.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct ProxyStatus {
    /// Value of git's global `http.proxy` (applies to both http and https
    /// URLs), or `None` when unset.
    pub git: Option<String>,
    /// Value of npm's `proxy` (used for http registries), or `None` when unset.
    pub npm: Option<String>,
    /// Value of npm's `https-proxy` (used for the default https registry), or
    /// `None` when unset.
    pub npm_https: Option<String>,
}

impl ProxyStatus {
    /// `true` when no proxy is configured for git or npm.
    ///
    /// # Examples
    ///
    /// ```
    /// use proxy_x::ProxyStatus;
    ///
    /// let disabled = ProxyStatus { git: None, npm: None, npm_https: None };
    /// assert!(disabled.is_disabled());
    ///
    /// let enabled = ProxyStatus { git: Some("http://x".into()), npm: None, npm_https: None };
    /// assert!(!enabled.is_disabled());
    /// ```
    pub fn is_disabled(&self) -> bool {
        self.git.is_none() && self.npm.is_none() && self.npm_https.is_none()
    }
}

/// Read the current proxy configuration from git and npm.
///
/// Read-only: this never modifies any config. A tool that is missing or
/// errors simply contributes `None` to the corresponding field, so this call
/// cannot fail. Used by the `status` command.
///
/// # Examples
///
/// ```no_run
/// let status = proxy_x::proxy_status();
/// if status.is_disabled() {
///     println!("no proxy configured");
/// }
/// ```
pub fn proxy_status() -> ProxyStatus {
    ProxyStatus {
        git: read_git_proxy(),
        npm: read_npm_config("proxy"),
        npm_https: read_npm_config("https-proxy"),
    }
}

/// Determine the local outbound IP address.
///
/// Connects an ephemeral UDP socket toward a public host (without sending any
/// data) to learn which local interface the kernel would use for outbound
/// traffic. Requires a usable network route.
///
/// # Examples
///
/// ```no_run
/// let ip = proxy_x::get_agent_ip().expect("no network route");
/// println!("local IP: {ip}");
/// ```
pub fn get_agent_ip() -> io::Result<String> {
    let socket = UdpSocket::bind(IP_DISCOVERY_BIND)?;
    socket.connect(IP_DISCOVERY_HOST)?;
    let local_addr = socket.local_addr()?;
    Ok(local_addr.ip().to_string())
}

/// Set or unset a configuration for a given tool.
///
/// Returns `Ok(())` on success, or an `io::Error` if the tool name is
/// invalid, the command could not be spawned, or the command exited with
/// a non-zero status.
fn set_config(key: &str, value: Option<&str>, tool: &str) -> io::Result<()> {
    let args = match (value, tool) {
        (Some(v), GIT) => vec!["config", "--global", key, v],
        (None, GIT) => vec!["config", "--global", "--unset", key],
        (Some(v), NPM) => vec!["config", "set", key, v],
        (None, NPM) => vec!["config", "delete", key],
        _ => {
            return Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                format!("Invalid tool: {}", tool),
            ))
        }
    };
    let output = Command::new(tool).args(&args).output()?;
    if !output.status.success() {
        // Unsetting a config key that doesn't exist is not an error.
        // git returns exit code 5 (GIT_CONFIG_KEY_NOT_FOUND) in this case.
        // npm's `config delete` returns exit code 0 for missing keys,
        // so only git needs this special handling. Treat git's exit 5 as
        // a no-op (the key was already absent).
        if value.is_none() && output.status.code() == Some(5) {
            return Ok(());
        }
        let stderr = String::from_utf8_lossy(&output.stderr);
        let stderr = stderr.trim();
        return Err(io::Error::other(format!(
            "Command '{}' failed with exit code {:?}: {}",
            tool,
            output.status.code(),
            stderr
        )));
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Mutex;

    /// Serialize tests that read/write the global git `http.proxy` key so
    /// parallel test threads don't race on the same config value.
    static CONFIG_LOCK: Mutex<()> = Mutex::new(());

    #[test]
    fn test_set_config_invalid_tool_returns_error() {
        let result = set_config("key", Some("value"), "nonexistent_tool");
        assert!(result.is_err(), "invalid tool should return an error");
        let err = result.unwrap_err();
        assert_eq!(
            err.kind(),
            io::ErrorKind::InvalidInput,
            "invalid tool should produce InvalidInput error"
        );
        assert!(
            err.to_string().contains("Invalid tool: nonexistent_tool"),
            "error message should contain the invalid tool name, got: {}",
            err
        );
    }

    #[test]
    fn test_set_config_git_unset_nonexistent_is_noop() {
        // Serialize with the other global-git-config writers: `git config
        // --global` takes an exclusive lock on the config file, so two
        // concurrent writes fail with a non-5 "could not lock config file"
        // error, which would break the no-op assertion below.
        let _guard = CONFIG_LOCK.lock().unwrap();

        // git returns exit code 5 (GIT_CONFIG_KEY_NOT_FOUND) when --unset is
        // used on a key that doesn't exist. set_config treats this as Ok(()).
        // Use an unlikely key name to avoid any risk of side effects.
        let result = set_config("test.proxy-x-nonexistent", None, GIT);
        assert!(
            result.is_ok(),
            "unsetting a non-existent git key should be a no-op (git exit 5), got: {:?}",
            result.err()
        );
    }

    #[test]
    fn test_set_config_command_failure_returns_error() {
        // An empty key is rejected by npm with a non-zero (exit code 1) status.
        // This exercises set_config's general error path: the command exits
        // non-zero, value is Some (so the git exit-5 no-op branch does not
        // apply), and a descriptive io::Error must be returned.
        let result = set_config("", Some("http://127.0.0.1:38080"), NPM);
        assert!(result.is_err(), "npm config set with an empty key should fail");
        let err = result.unwrap_err().to_string();
        assert!(
            err.contains(NPM),
            "error message should mention the tool name '{NPM}', got: {err}"
        );
        assert!(
            err.contains("exit code"),
            "error message should report the exit code, got: {err}"
        );
    }

    #[test]
    fn test_validate_proxy_url_accepts_schemed_urls() {
        // Every valid proxy URL form includes a scheme separator ("://"),
        // including userinfo and socks variants.
        for url in [
            "http://127.0.0.1:8080",
            "https://proxy.example.com:443",
            "socks5://10.0.0.1:1080",
            "socks5h://user:pass@host:1080",
        ] {
            assert!(
                validate_proxy_url(url).is_ok(),
                "expected valid proxy URL to pass: {url}"
            );
        }
    }

    #[test]
    fn test_validate_proxy_url_rejects_scheme_less_and_empty() {
        // git and npm silently accept scheme-less URLs at config-set time, but
        // a scheme-less proxy is unusable by git and only warned-on by npm.
        // validate_proxy_url must fail fast instead of storing broken config.
        for bad in ["", "   ", "localhost:8080", "127.0.0.1:8080", "not a url"] {
            let result = validate_proxy_url(bad);
            assert!(result.is_err(), "expected invalid proxy URL to be rejected: {bad:?}");
            let msg = result.unwrap_err().to_string();
            assert!(
                msg.contains("scheme") || msg.contains("empty"),
                "error should mention the scheme requirement or emptiness, got: {msg}"
            );
        }
    }

    #[test]
    fn test_read_git_proxy_round_trips_config_value() {
        let _guard = CONFIG_LOCK.lock().unwrap();

        // Start from a clean slate so the assertions are deterministic.
        let _ = set_config("http.proxy", None, GIT);
        assert_eq!(
            read_git_proxy(),
            None,
            "read_git_proxy should be None when the key is unset"
        );

        // Set a value and confirm it is read back exactly.
        let url = "http://127.0.0.1:38080";
        set_config("http.proxy", Some(url), GIT).expect("setting git proxy should succeed");
        assert_eq!(
            read_git_proxy().as_deref(),
            Some(url),
            "read_git_proxy should return the value just written"
        );

        // Unset and confirm it returns to None (also serves as cleanup).
        let _ = set_config("http.proxy", None, GIT);
        assert_eq!(read_git_proxy(), None, "read_git_proxy should be None after unsetting");
    }

    #[test]
    fn test_parse_npm_config_output() {
        assert_eq!(
            parse_npm_config_output("http://127.0.0.1:8080\n"),
            Some("http://127.0.0.1:8080".to_string()),
            "a set value should be returned trimmed"
        );
        assert_eq!(
            parse_npm_config_output("undefined\n"),
            None,
            "older npm prints the literal 'undefined' for unset keys"
        );
        assert_eq!(
            parse_npm_config_output("null\n"),
            None,
            "newer npm prints the literal 'null' for unset keys"
        );
        assert_eq!(parse_npm_config_output(""), None, "empty output means unset");
        assert_eq!(
            parse_npm_config_output("   \n"),
            None,
            "whitespace-only output means unset"
        );
    }

    #[test]
    fn test_proxy_status_is_disabled() {
        assert!(ProxyStatus {
            git: None,
            npm: None,
            npm_https: None
        }
        .is_disabled());
        assert!(!ProxyStatus {
            git: Some("http://x".to_string()),
            npm: None,
            npm_https: None
        }
        .is_disabled());
        assert!(!ProxyStatus {
            git: None,
            npm: Some("http://x".to_string()),
            npm_https: None
        }
        .is_disabled());
        assert!(!ProxyStatus {
            git: None,
            npm: None,
            npm_https: Some("http://x".to_string())
        }
        .is_disabled());
    }

    #[test]
    fn test_proxy_status_reflects_git_config() {
        let _guard = CONFIG_LOCK.lock().unwrap();

        let _ = set_config("http.proxy", None, GIT);
        assert_eq!(proxy_status().git, None, "git proxy should be None when unset");

        let url = "http://127.0.0.1:38080";
        set_config("http.proxy", Some(url), GIT).expect("setting git proxy should succeed");
        assert_eq!(
            proxy_status().git.as_deref(),
            Some(url),
            "proxy_status should report the git proxy just written"
        );

        let _ = set_config("http.proxy", None, GIT);
        assert_eq!(proxy_status().git, None, "cleanup should clear the git proxy");
    }

    /// A non-existent npm binary makes every npm step fail, letting these tests
    /// exercise the rollback path (git set, then npm fails) without needing npm
    /// to genuinely break.
    const BAD_NPM: &str = "proxy-x-nonexistent-npm-binary";

    #[test]
    fn test_enable_proxy_rolls_back_git_to_prior_value_when_npm_fails() {
        let _guard = CONFIG_LOCK.lock().unwrap();

        // Establish a pre-existing git proxy that a failed enable must preserve.
        let prior = "http://prior.proxy.example:1111";
        set_config("http.proxy", Some(prior), GIT).expect("setting prior git proxy");

        let result = enable_proxy_with("http://new.proxy.example:2222", GIT, BAD_NPM);
        assert!(result.is_err(), "enable should fail when npm is unavailable");

        // Core guarantee: git http.proxy is restored to the PRIOR value — not
        // wiped, and not left at the new value.
        assert_eq!(
            read_git_proxy().as_deref(),
            Some(prior),
            "git http.proxy must be rolled back to its prior value on npm failure"
        );

        let _ = set_config("http.proxy", None, GIT);
    }

    #[test]
    fn test_enable_proxy_unsets_git_when_npm_fails_and_no_prior() {
        let _guard = CONFIG_LOCK.lock().unwrap();

        let _ = set_config("http.proxy", None, GIT);
        assert_eq!(read_git_proxy(), None, "precondition: no prior git proxy");

        let result = enable_proxy_with("http://new.proxy.example:2222", GIT, BAD_NPM);
        assert!(result.is_err(), "enable should fail when npm is unavailable");

        // With no prior value, the rollback must unset git (not leave the new
        // value behind).
        assert_eq!(
            read_git_proxy(),
            None,
            "git http.proxy must be unset on rollback when there was no prior value"
        );
    }

    #[test]
    fn test_disable_proxy_restores_git_when_npm_fails() {
        let _guard = CONFIG_LOCK.lock().unwrap();

        let prior = "http://prior.proxy.example:3333";
        set_config("http.proxy", Some(prior), GIT).expect("setting prior git proxy");

        let result = disable_proxy_with(GIT, BAD_NPM);
        assert!(result.is_err(), "disable should fail when npm is unavailable");

        // The git proxy must be restored after a failed disable.
        assert_eq!(
            read_git_proxy().as_deref(),
            Some(prior),
            "git http.proxy must be restored on a failed disable"
        );

        let _ = set_config("http.proxy", None, GIT);
    }
}
// verified integration
// v2
// auto-hook test
// test2
fn bad() {}
// prek-native test
fn bad() {}
// clean test
