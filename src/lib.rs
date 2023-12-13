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
