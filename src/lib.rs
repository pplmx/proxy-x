use std::io;
use std::net::UdpSocket;
use std::process::Command;

pub mod pin;
pub mod ping;

#[cfg(windows)]
pub const NPM: &str = "npm.cmd";

#[cfg(not(windows))]
pub const NPM: &str = "npm";

pub fn enable_proxy(proxy_url: &str) -> io::Result<()> {
    set_config("http.proxy", Some(proxy_url), "git")?;
    set_config("proxy", Some(proxy_url), NPM)?;
    println!("Proxy enabled");
    Ok(())
}

pub fn disable_proxy() -> io::Result<()> {
    set_config("http.proxy", None, "git")?;
    set_config("proxy", None, NPM)?;
    println!("Proxy disabled");
    Ok(())
}

pub fn get_agent_ip() -> io::Result<String> {
    let socket = UdpSocket::bind("0.0.0.0:0")?;
    socket.connect("8.8.8.8:53")?;
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
        (Some(v), "git") => vec!["config", "--global", key, v],
        (None, "git") => vec!["config", "--global", "--unset", key],
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
        // git returns exit code 5 (GIT_CONFIG_KEY_NOT_FOUND) in this case;
        // npm returns a non-zero exit code too. Treat it as a no-op.
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
