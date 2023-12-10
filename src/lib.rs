// lib.rs
pub mod proxy_manager {
    use std::process::Command;

    pub fn enable_proxy(proxy_url: &str) {
        // git config
        git_set_config("http.proxy", proxy_url);

        // npm config
        npm_set_config("proxy", proxy_url);

        // export some proxy environment variables
        std::env::set_var("all_proxy", proxy_url);
        std::env::set_var("http_proxy", proxy_url);
        std::env::set_var("https_proxy", proxy_url);
        std::env::set_var("no_proxy", "localhost, 127.0.0.1, ::1, .local, .internal, 192.168.0.0/16, 10.0.0.0/8, 172.16.0.0/12");
        println!("Proxy enabled");
    }

    pub fn disable_proxy() {
        git_unset_config("http.proxy");

        npm_unset_config("proxy");

        std::env::remove_var("all_proxy");
        std::env::remove_var("http_proxy");
        std::env::remove_var("https_proxy");
        std::env::remove_var("no_proxy");
        println!("Proxy disabled");
    }

    fn git_set_config(key: &str, value: &str) {
        Command::new("git")
            .args(&["config", "--global", key, value])
            .output()
            .ok();
    }

    fn git_unset_config(key: &str) {
        Command::new("git")
            .args(&["config", "--global", "--unset", key])
            .output()
            .ok();
    }

    fn npm_set_config(key: &str, value: &str) {
        if cfg!(target_os = "windows") {
            Command::new("npm.cmd")
                .args(&["config", "set", key, value])
                .output()
                .ok();
        } else {
            Command::new("npm")
                .args(&["config", "set", key, value])
                .output()
                .ok();
        }
    }

    fn npm_unset_config(key: &str) {
        if cfg!(target_os = "windows") {
            Command::new("npm.cmd")
                .args(&["config", "delete", key])
                .output()
                .ok();
        } else {
            Command::new("npm")
                .args(&["config", "delete", key])
                .output()
                .ok();
        }
    }
}
