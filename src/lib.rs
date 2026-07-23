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

pub fn enable_proxy(proxy_url: &str) -> io::Result<()> {
    set_config("http.proxy", Some(proxy_url), GIT)?;
    if let Err(e) = set_config("proxy", Some(proxy_url), NPM) {
        // Rollback: git proxy was set successfully, but npm failed.
        // Attempt to unset the git proxy to avoid leaving a partial state.
        let _ = set_config("http.proxy", None, GIT);
        return Err(e);
    }
    println!("Proxy enabled");
    Ok(())
}

pub fn disable_proxy() -> io::Result<()> {
    // Save the current git proxy value so we can roll back if npm fails.
    let git_proxy_before = Command::new(GIT)
        .args(["config", "--global", "http.proxy"])
        .output()
        .ok()
        .and_then(|o| {
            if o.status.success() {
                Some(String::from_utf8_lossy(&o.stdout).trim().to_string())
            } else {
                None
            }
        });

    set_config("http.proxy", None, GIT)?;
    if let Err(e) = set_config("proxy", None, NPM) {
        // Rollback: git proxy was unset, but npm failed.
        // Attempt to restore the previous git proxy value.
        if let Some(prev) = git_proxy_before {
            let _ = set_config("http.proxy", Some(&prev), GIT);
        }
        return Err(e);
    }
    println!("Proxy disabled");
    Ok(())
}

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
}
