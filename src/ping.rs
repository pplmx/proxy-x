use std::io;
use std::process::Command;

/// Parameters for a synchronous ICMP ping.
///
/// These map directly to the `ping` CLI options.
#[derive(Clone, Debug)]
pub struct PingParams<'a> {
    pub destination: &'a str,
    pub count: u64,
    pub size: usize,
    pub ttl: u32,
    pub interval: u64,
}

/// Send ICMP ECHO_REQUEST to network hosts by invoking the system `ping`
/// command.
///
/// This approach is used instead of `pnet` (raw sockets) because:
/// - Raw sockets typically require root/cap_net_raw privileges.
/// - The system `ping` is already setuid or has the necessary capabilities.
/// - It avoids an additional dependency and cross-platform raw-socket
///   differences.
///
/// On Linux and macOS the full parameter set is passed through. On Windows
/// only `count` and `size` are mapped (TTL via `-i` and interval are not
/// supported by the built-in Windows `ping`).
///
/// # Examples
///
/// ```no_run
/// use proxy_x::ping::{ping, PingParams};
///
/// let params = PingParams {
///     destination: "example.com",
///     count: 4,
///     size: 56,
///     ttl: 64,
///     interval: 1000,
/// };
/// ping(&params).expect("ping failed");
/// ```
pub fn ping(params: &PingParams) -> io::Result<()> {
    execute_ping(ping_binary(), params)
}

/// Execute a ping using the given binary name.
///
/// Separated from [`ping`] so tests can inject a non-existent binary and
/// observe the spawn-failure error (with its [`io::ErrorKind`] preserved).
fn execute_ping(binary: &str, params: &PingParams) -> io::Result<()> {
    let mut cmd = Command::new(binary);
    build_args(&mut cmd, params);

    // Propagate spawn errors (e.g. a missing binary) directly so the original
    // io::ErrorKind (such as NotFound) is preserved for callers that match on
    // it, rather than being flattened to ErrorKind::Other.
    let output = cmd.output()?;

    // Print the system ping output so the user sees per-packet results.
    if !output.stdout.is_empty() {
        print!("{}", String::from_utf8_lossy(&output.stdout));
    }

    if output.status.success() {
        Ok(())
    } else {
        let stderr = String::from_utf8_lossy(&output.stderr);
        let stderr = stderr.trim();
        let detail = if stderr.is_empty() {
            // ping may report errors on stdout rather than stderr.
            let stdout = String::from_utf8_lossy(&output.stdout);
            stdout.trim().to_string()
        } else {
            stderr.to_string()
        };
        Err(io::Error::other(format!(
            "ping to {} failed with exit code {:?}: {}",
            params.destination,
            output.status.code(),
            detail
        )))
    }
}

/// Returns the platform-appropriate ping binary name.
fn ping_binary() -> &'static str {
    // On Windows the binary is `ping` (not `ping.exe` — the shell resolves
    // it). On Unix it is `/bin/ping` or resolved via PATH.
    "ping"
}

/// Build the argument list for the current platform.
#[cfg(not(windows))]
fn build_args(cmd: &mut Command, params: &PingParams) {
    cmd.args(["-c", &params.count.to_string(), "-s", &params.size.to_string()]);

    // TTL
    cmd.arg("-t");
    cmd.arg(params.ttl.to_string());

    // Interval: -i expects seconds (float). On Linux, values < 1 require
    // elevated privileges, but the default 1000ms (1.0s) is fine.
    let interval_s = params.interval as f64 / 1000.0;
    cmd.arg("-i");
    cmd.arg(format!("{interval_s}"));

    // Per-reply timeout (-W) prevents hanging indefinitely on unreachable
    // hosts. 5 seconds is a reasonable default that matches system behavior.
    cmd.arg("-W");
    cmd.arg("5");

    cmd.arg(params.destination);
}

/// Build the argument list for Windows.
#[cfg(windows)]
fn build_args(cmd: &mut Command, params: &PingParams) {
    cmd.args(["-n", &params.count.to_string(), "-l", &params.size.to_string()]);
    // -w sets the per-reply timeout in milliseconds. 5000ms (5s) prevents
    // hanging on unreachable hosts.
    cmd.arg("-w");
    cmd.arg("5000");
    cmd.arg(params.destination);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ping_params_fields() {
        let params = PingParams {
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
    fn test_ping_successful() {
        // Ping localhost — should always succeed if the system has a
        // loopback interface and the ping binary is available.
        let params = PingParams {
            destination: "127.0.0.1",
            count: 1,
            size: 56,
            ttl: 64,
            interval: 1000,
        };
        let result = ping(&params);
        assert!(result.is_ok(), "ping to 127.0.0.1 should succeed: {:?}", result.err());
    }

    #[test]
    fn test_ping_fails_on_invalid_destination() {
        // An invalid hostname should cause ping to fail (exit non-zero).
        let params = PingParams {
            destination: "this-host-definitely-does-not-exist.invalid",
            count: 1,
            size: 56,
            ttl: 64,
            interval: 1000,
        };
        let result = ping(&params);
        assert!(result.is_err(), "ping to an invalid host should fail");
        let err = result.unwrap_err().to_string();
        assert!(
            err.contains("this-host-definitely-does-not-exist"),
            "error should mention the destination: {err}"
        );
    }

    #[test]
    fn test_execute_ping_missing_binary_preserves_not_found_kind() {
        // A non-existent binary makes Command::output() fail with
        // ErrorKind::NotFound. execute_ping must propagate that error WITHOUT
        // re-wrapping it (the old `.map_err(io::Error::other)` flattened the
        // kind to Other), so callers can still match on ErrorKind::NotFound.
        let params = PingParams {
            destination: "127.0.0.1",
            count: 1,
            size: 56,
            ttl: 64,
            interval: 1000,
        };
        let result = execute_ping("proxy-x-nonexistent-ping-binary", &params);
        let err = result.expect_err("a missing ping binary should error");
        assert_eq!(
            err.kind(),
            io::ErrorKind::NotFound,
            "spawn failure should preserve ErrorKind::NotFound, got: {err:?}"
        );
    }
}
