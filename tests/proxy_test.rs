// Tests for proxy-x public API.
//
// Note: enable_proxy and disable_proxy modify global git/npm config
// as a side effect. Tests clean up after themselves by calling
// disable_proxy at the end.

use std::sync::Mutex;

/// Mutex to serialize tests that modify global git/npm config.
/// Without this, parallel test threads race on the same config keys.
static CONFIG_LOCK: Mutex<()> = Mutex::new(());

/// Locate the freshly-built `proxy-x` binary.
///
/// `CARGO_BIN_EXE_*` is not reliably injected by every Cargo configuration, so
/// resolve the binary via `CARGO_MANIFEST_DIR` (falling back to the current
/// directory, which is the package root when `cargo test` runs integration
/// tests) joined with the debug profile output.
fn cargo_bin() -> std::path::PathBuf {
    let base = std::env::var("CARGO_MANIFEST_DIR")
        .ok()
        .map(std::path::PathBuf::from)
        .unwrap_or_else(|| std::path::PathBuf::from("."));
    let bin = base.join("target").join("debug").join("proxy-x");
    assert!(
        bin.exists(),
        "proxy-x binary not found at {bin:?}; run `cargo build` first"
    );
    bin
}

/// Returns Ok(()) if the `git` binary is available, Err otherwise.
fn git_available() -> std::io::Result<()> {
    std::process::Command::new("git").arg("--version").output().map(|o| {
        if o.status.success() {
            Ok(())
        } else {
            Err(std::io::Error::other("git --version failed"))
        }
    })?
}

/// Read an npm config value by key, returning the trimmed stdout.
///
/// For an unset key npm prints a sentinel (`null` on newer versions,
/// `undefined` on older), so callers compare against those rather than empty.
fn npm_config_get(key: &str) -> String {
    let output = std::process::Command::new("npm")
        .args(["config", "get", key])
        .output()
        .expect("failed to read npm config");
    String::from_utf8_lossy(&output.stdout).trim().to_string()
}

#[test]
fn test_enable_disable_proxy_cycle() {
    let _guard = CONFIG_LOCK.lock().unwrap();
    // Clean up any leftover state from previous runs
    let _ = proxy_x::disable_proxy();

    // Skip if git or npm is not available (e.g., CI without them)
    if git_available().is_err() {
        eprintln!("Skipping: git/npm not available");
        return;
    }

    let proxy_url = "http://127.0.0.1:38080";

    // Enable proxy should succeed and actually set the config
    let enable_result = proxy_x::enable_proxy(proxy_url);
    assert!(
        enable_result.is_ok(),
        "enable_proxy should succeed when git/npm are available: {:?}",
        enable_result.err()
    );

    // Verify git config was actually set
    let git_config = std::process::Command::new("git")
        .args(["config", "--global", "http.proxy"])
        .output()
        .expect("failed to read git config");
    assert!(git_config.status.success());
    let git_val = String::from_utf8_lossy(&git_config.stdout).trim().to_string();
    assert_eq!(git_val, proxy_url, "git http.proxy should be set to the proxy URL");

    // Verify BOTH npm keys were set. The default npm registry is https, so
    // enable_proxy must set `https-proxy` (used for https registries) in
    // addition to `proxy` (used for http registries) for `npm install` to
    // actually route through the proxy.
    assert_eq!(
        npm_config_get("proxy"),
        proxy_url,
        "npm proxy should be set to the proxy URL"
    );
    assert_eq!(
        npm_config_get("https-proxy"),
        proxy_url,
        "npm https-proxy should be set to the proxy URL"
    );

    // Disable proxy should succeed and clear the config
    let disable_result = proxy_x::disable_proxy();
    assert!(
        disable_result.is_ok(),
        "disable_proxy should succeed: {:?}",
        disable_result.err()
    );

    // Verify git config was actually unset
    let git_config_after = std::process::Command::new("git")
        .args(["config", "--global", "http.proxy"])
        .output()
        .expect("failed to read git config");
    // git config --unset leaves the key absent; --get exits non-zero
    assert!(
        !git_config_after.status.success(),
        "git http.proxy should be unset after disable_proxy"
    );

    // Verify BOTH npm keys were cleared. npm prints "null" (newer) or
    // "undefined" (older) for an unset key.
    for key in ["proxy", "https-proxy"] {
        let value = npm_config_get(key);
        assert!(
            matches!(value.as_str(), "null" | "undefined"),
            "npm {key} should be unset after disable_proxy, got: {value}"
        );
    }
}

#[test]
fn test_get_agent_ip_returns_valid_ipv4_or_skips() {
    // get_agent_ip connects to 8.8.8.8:53 to determine the local IP.
    // This requires network access, so we skip if it fails.
    let result = proxy_x::get_agent_ip();
    match result {
        Ok(ip) => {
            // On success, the IP must be a valid IPv4 address
            let parsed: std::result::Result<std::net::Ipv4Addr, _> = ip.parse();
            assert!(
                parsed.is_ok(),
                "get_agent_ip should return a valid IPv4 address, got: {}",
                ip
            );
            // Loopback would indicate the socket didn't actually connect
            let addr = parsed.unwrap();
            assert!(!addr.is_loopback(), "get_agent_ip should not return a loopback address");
        }
        Err(e) => {
            eprintln!("get_agent_ip failed (expected in restricted network): {}", e);
        }
    }
}

#[test]
fn test_disable_proxy_is_idempotent_when_not_set() {
    let _guard = CONFIG_LOCK.lock().unwrap();
    // When no proxy is currently set, disable_proxy should still succeed
    // instead of failing with a git exit code 5 (GIT_CONFIG_KEY_NOT_FOUND).
    if git_available().is_err() {
        eprintln!("Skipping: git/npm not available");
        return;
    }

    // Ensure no proxy is set
    let _ = proxy_x::disable_proxy();

    // Calling disable again should not fail even though the key doesn't exist
    let result = proxy_x::disable_proxy();
    assert!(
        result.is_ok(),
        "disable_proxy should be idempotent (succeed when proxy not set): {:?}",
        result.err()
    );
}

#[test]
fn test_ping_successful_to_localhost() {
    // ping to 127.0.0.1 should succeed if the system has a loopback
    // interface and the `ping` binary is available.
    let params = proxy_x::ping::PingParams {
        destination: "127.0.0.1",
        count: 1,
        size: 56,
        ttl: 64,
        interval: 1000,
    };
    let result = proxy_x::ping::ping(&params);
    assert!(result.is_ok(), "ping to 127.0.0.1 should succeed: {:?}", result.err());
}

#[test]
fn test_ping_fails_on_invalid_destination() {
    let params = proxy_x::ping::PingParams {
        destination: "this-host-definitely-does-not-exist.invalid",
        count: 1,
        size: 56,
        ttl: 64,
        interval: 1000,
    };
    let result = proxy_x::ping::ping(&params);
    assert!(result.is_err(), "ping to invalid host should fail");
    let err = result.unwrap_err().to_string();
    assert!(
        err.contains("this-host-definitely-does-not-exist"),
        "error should mention destination: {err}"
    );
}

#[test]
fn test_async_ping_successful_to_localhost() {
    let params = proxy_x::pin::PingParams {
        destination: "127.0.0.1",
        count: 1,
        size: 56,
        ttl: 64,
        interval: 1000,
    };
    let result = proxy_x::pin::async_ping(&params);
    assert!(
        result.is_ok(),
        "async_ping to 127.0.0.1 should succeed: {:?}",
        result.err()
    );
}

#[test]
fn test_cli_exits_nonzero_on_failure_and_reports_stderr() {
    // End-to-end: a failing command (`enable` with a scheme-less URL) must
    // surface a message on stderr AND exit with a non-zero status so that
    // shells and CI can detect the failure. Validation runs before any
    // git/npm config write, so this test must not mutate global config.
    let bin = cargo_bin();
    let output = std::process::Command::new(bin)
        .args(["enable", "localhost:8080"])
        .output()
        .expect("failed to execute proxy-x binary");

    assert!(
        !output.status.success(),
        "failing command should exit non-zero; got {:?}",
        output.status
    );

    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        stderr.contains("Error enabling proxy"),
        "stderr should report the proxied backend error, got: {stderr}"
    );
    assert!(
        stderr.contains("scheme"),
        "stderr should mention the scheme requirement, got: {stderr}"
    );

    // Validation runs before any config write, so git's http.proxy must be
    // absent after a rejected enable.
    let git_proxy = std::process::Command::new("git")
        .args(["config", "--global", "http.proxy"])
        .output()
        .expect("failed to read git config");
    assert!(
        !git_proxy.status.success(),
        "git http.proxy must not be set after a rejected enable"
    );
}

#[test]
fn test_ping_params_fields() {
    // Verify PingParams can be constructed with all CLI argument values
    let params = proxy_x::ping::PingParams {
        destination: "test.host",
        count: 10,
        size: 256,
        ttl: 255,
        interval: 2000,
    };
    assert_eq!(params.destination, "test.host");
    assert_eq!(params.count, 10);
    assert_eq!(params.size, 256);
    assert_eq!(params.ttl, 255);
    assert_eq!(params.interval, 2000);
}

#[test]
fn test_cli_status_reflects_proxy_state_end_to_end() {
    let _guard = CONFIG_LOCK.lock().unwrap();
    if git_available().is_err() {
        eprintln!("Skipping: git/npm not available");
        return;
    }
    let bin = cargo_bin();
    let proxy_url = "http://127.0.0.1:38080";

    // Start from a clean (disabled) state.
    let _ = proxy_x::disable_proxy();

    // `status` should report disabled when no proxy is set.
    let output = std::process::Command::new(&bin)
        .arg("status")
        .output()
        .expect("failed to execute proxy-x binary");
    assert!(output.status.success(), "status should exit 0");
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        stdout.contains("Proxy is disabled"),
        "status should report disabled when no proxy is set, got: {stdout}"
    );

    // After enabling, `status` should report the proxy URL.
    proxy_x::enable_proxy(proxy_url).expect("enable_proxy should succeed");
    let output = std::process::Command::new(&bin)
        .arg("status")
        .output()
        .expect("failed to execute proxy-x binary");
    assert!(output.status.success(), "status should exit 0");
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        stdout.contains(proxy_url),
        "status should report the enabled proxy URL, got: {stdout}"
    );

    // Cleanup.
    let _ = proxy_x::disable_proxy();
}
