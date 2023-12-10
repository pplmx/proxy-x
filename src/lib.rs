// lib.rs
#[cfg(windows)]
pub const NPM: &str = "npm.cmd";

#[cfg(not(windows))]
pub const NPM: &str = "npm";

pub mod proxy_manager {
    use std::process::Command;

    use crate::NPM;

    pub fn enable_proxy(proxy_url: &str) {
        // git config
        git_config("http.proxy", Some(proxy_url));

        // npm config
        npm_config("proxy", Some(proxy_url));

        // export some proxy environment variables
        std::env::set_var("all_proxy", proxy_url);
        std::env::set_var("http_proxy", proxy_url);
        std::env::set_var("https_proxy", proxy_url);
        std::env::set_var("no_proxy", "localhost, 127.0.0.1, ::1, .local, .internal, 192.168.0.0/16, 10.0.0.0/8, 172.16.0.0/12");
        println!("Proxy enabled");
    }

    pub fn disable_proxy() {
        git_config("http.proxy", None);

        npm_config("proxy", None);

        std::env::remove_var("all_proxy");
        std::env::remove_var("http_proxy");
        std::env::remove_var("https_proxy");
        std::env::remove_var("no_proxy");
        println!("Proxy disabled");
    }

    // git global config. if value is None, unset the key
    fn git_config(key: &str, value: Option<&str>) {
        let mut args = vec!["config", "--global"];
        match value {
            Some(v) => args.extend_from_slice(&[key, v]),
            None => args.extend_from_slice(&["--unset", key]),
        }
        Command::new("git").args(&args).output().ok();
    }

    // npm config. if value is None, unset the key
    fn npm_config(key: &str, value: Option<&str>) {
        let mut args = vec!["config"];
        match value {
            Some(v) => args.extend_from_slice(&[key, v]),
            None => args.extend_from_slice(&["delete", key]),
        }
        Command::new(NPM).args(&args).output().ok();
    }
}
