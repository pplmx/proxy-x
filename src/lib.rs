use std::io;
use std::net::UdpSocket;
use std::process::Command;

#[cfg(windows)]
pub const NPM: &str = "npm.cmd";

#[cfg(not(windows))]
pub const NPM: &str = "npm";

pub fn enable_proxy(proxy_url: &str) {
    set_config("http.proxy", Some(proxy_url), "git");
    set_config("proxy", Some(proxy_url), NPM);
    println!("Proxy enabled");
}

pub fn disable_proxy() {
    set_config("http.proxy", None, "git");
    set_config("proxy", None, NPM);
    println!("Proxy disabled");
}

pub fn get_agent_ip() -> io::Result<String> {
    let socket = UdpSocket::bind("0.0.0.0:0")?;
    socket.connect("8.8.8.8:53")?;

    let local_addr = socket.local_addr()?;
    Ok(local_addr.ip().to_string())
}

// Set or unset a configuration for a given tool.
fn set_config(key: &str, value: Option<&str>, tool: &str) {
    let mut args = match value {
        Some(v) => vec!["config", "--global", key, v],
        None => vec!["config", "--global", "--unset", key],
    };
    if tool == NPM {
        args = match value {
            Some(v) => vec!["config", "set", key, v],
            None => vec!["config", "delete", key],
        };
    }
    if let Err(e) = Command::new(tool).args(&args).output() {
        eprintln!("Failed to set config for {}: {}", tool, e);
    }
}
