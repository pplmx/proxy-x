use std::env;
use std::process::Command;

#[cfg(windows)]
pub const NPM: &str = "npm.cmd";

#[cfg(not(windows))]
pub const NPM: &str = "npm";

pub fn enable_proxy(proxy_url: &str) {
    set_config("http.proxy", Some(proxy_url), "git");
    set_config("proxy", Some(proxy_url), NPM);

    let proxies = ["all_proxy", "http_proxy", "https_proxy"];
    let no_proxy = "localhost, 127.0.0.1, ::1, .local, .internal, 192.168.0.0/16, 10.0.0.0/8, 172.16.0.0/12";
    for proxy in proxies.iter() {
        env::set_var(proxy, proxy_url);
    }
    env::set_var("no_proxy", no_proxy);
    println!("Proxy enabled");
}

pub fn disable_proxy() {
    set_config("http.proxy", None, "git");
    set_config("proxy", None, NPM);

    let proxies = ["all_proxy", "http_proxy", "https_proxy", "no_proxy"];
    for proxy in proxies.iter() {
        env::remove_var(proxy);
    }
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
