// lib.rs
pub mod proxy_manager {
    use std::process::Command;

    pub fn enable_proxy(proxy_url: &str) {
        set_config("http.proxy", proxy_url);
        set_config("https.proxy", proxy_url);
        set_env("all_proxy", proxy_url);
        set_env("http_proxy", proxy_url);
        set_env("https_proxy", proxy_url);
        println!("Proxy enabled");
    }

    pub fn disable_proxy() {
        unset_config("http.proxy");
        unset_config("https.proxy");
        unset_env("all_proxy");
        unset_env("http_proxy");
        unset_env("https_proxy");
        println!("Proxy disabled");
    }

    fn set_config(key: &str, value: &str) {
        Command::new("git")
            .args(&["config", "--global", key, value])
            .output()
            .expect("Failed to execute command");
        Command::new("npm")
            .args(&["config", "set", "proxy", value])
            .output()
            .expect("Failed to execute command");
    }

    fn unset_config(key: &str) {
        Command::new("git")
            .args(&["config", "--global", "--unset", key])
            .output()
            .expect("Failed to execute command");
        Command::new("npm")
            .args(&["config", "delete", "proxy"])
            .output()
            .expect("Failed to execute command");
    }

    fn set_env(key: &str, value: &str) {
        std::env::set_var(key, value);
    }

    fn unset_env(key: &str) {
        std::env::remove_var(key);
    }
}
