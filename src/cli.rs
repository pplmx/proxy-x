use clap::{Args, Parser, Subcommand};

use proxy_x::{disable_proxy, enable_proxy, get_agent_ip, pin::async_ping, ping::ping};

#[derive(Parser)]
#[command(arg_required_else_help = true, author, version, about, long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Option<Commands>,
}

#[derive(Subcommand)]
enum Commands {
    #[command(about = "Get the current IP address")]
    Ip,
    #[command(about = "Enable proxy")]
    Enable(EnableArgs),
    #[command(about = "Disable proxy")]
    Disable,
    #[command(about = "Send ICMP ECHO_REQUEST to network hosts.")]
    Ping(PingArgs),
    #[command(about = "Send ICMP ECHO_REQUEST to network hosts (alternate).")]
    Pin(PingArgs),
}

#[derive(Args)]
struct EnableArgs {
    #[arg(required = true)]
    proxy_url: String,
}

#[derive(Args)]
struct PingArgs {
    #[arg(required = true, help = "dns name or ip address")]
    destination: String,

    #[arg(short, default_value_t = 4, help = "Stop after <count> replies")]
    count: u8,

    #[arg(
        short,
        default_value_t = 56,
        help = "Use <size> as number of data bytes to be sent"
    )]
    size: usize,

    #[arg(short, default_value_t = 64, help = "Define time to live")]
    ttl: u32,

    #[arg(
        short,
        default_value_t = 1000,
        help = "Wait <interval> milliseconds between sending each packet"
    )]
    interval: u64,
}

/// Parse and execute the CLI command.
///
/// On a command failure, the error is printed to stderr and the process exits
/// with status `1` so that shells, `&&`, `$?`, and CI can detect the failure.
/// Successful commands exit with status `0`. (Argument-parsing errors are
/// handled by `clap`, which exits with its own non-zero status.)
pub fn execute() {
    let cli = Cli::parse();
    if let Some(msg) = run(cli.command) {
        eprintln!("{msg}");
        std::process::exit(1);
    }
}

/// Dispatch a parsed CLI command to the appropriate backend function.
///
/// Returns `Some(message)` when the backend call fails, so that [`execute`]
/// can print it to stderr. Returns `None` on success or when there is
/// nothing to report.
///
/// Separated from [`execute`] so it can be unit-tested without going through
/// `clap`'s `Cli::parse()`, which reads `std::env::args()`.
fn run(command: Option<Commands>) -> Option<String> {
    match command {
        Some(Commands::Ip) => match get_agent_ip() {
            Ok(ip) => println!("Current IP: {}", ip),
            Err(e) => return Some(format!("Error getting IP: {e}")),
        },
        Some(Commands::Enable(args)) => match enable_proxy(&args.proxy_url) {
            Ok(()) => {}
            Err(e) => return Some(format!("Error enabling proxy: {e}")),
        },
        Some(Commands::Disable) => match disable_proxy() {
            Ok(()) => {}
            Err(e) => return Some(format!("Error disabling proxy: {e}")),
        },
        Some(Commands::Ping(args)) => {
            let params = proxy_x::ping::PingParams {
                destination: &args.destination,
                count: args.count,
                size: args.size,
                ttl: args.ttl,
                interval: args.interval,
            };
            match ping(&params) {
                Ok(()) => println!("Ping to {} completed", args.destination),
                Err(e) => return Some(format!("Error: {e}")),
            }
        }
        Some(Commands::Pin(args)) => {
            let params = proxy_x::pin::PingParams {
                destination: &args.destination,
                count: args.count,
                size: args.size,
                ttl: args.ttl,
                interval: args.interval,
            };
            match async_ping(&params) {
                Ok(()) => println!("Ping to {} completed", args.destination),
                Err(e) => return Some(format!("Error: {e}")),
            }
        }
        None => {}
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Mutex;

    /// Serialize tests that mutate global git/npm config.
    ///
    /// `run(Enable)` and `run(Disable)` invoke `enable_proxy`/`disable_proxy`,
    /// which read and write the global git and npm config. Running them in
    /// parallel would race on the same config keys, so they take this lock.
    static CONFIG_LOCK: Mutex<()> = Mutex::new(());

    /// Returns `Ok(())` if the `git` binary is usable, `Err` otherwise.
    ///
    /// Tests that depend on git/npm are skipped (not failed) when the toolchain
    /// is absent, e.g. in minimal CI images.
    fn git_available() -> std::io::Result<()> {
        std::process::Command::new(proxy_x::GIT)
            .arg("--version")
            .output()
            .map(|o| {
                if o.status.success() {
                    Ok(())
                } else {
                    Err(std::io::Error::other("git --version failed"))
                }
            })?
    }

    /// Helper to construct a `PingArgs` with sensible defaults.
    fn default_ping_args(destination: &str) -> PingArgs {
        PingArgs {
            destination: destination.to_string(),
            count: 4,
            size: 56,
            ttl: 64,
            interval: 1000,
        }
    }

    /// Helper to construct an `EnableArgs` for a local test proxy.
    fn test_enable_args() -> EnableArgs {
        EnableArgs {
            proxy_url: "http://127.0.0.1:38080".to_string(),
        }
    }

    #[test]
    fn test_run_none_returns_none() {
        // Running without a subcommand is a no-op — no error message.
        assert!(run(None).is_none());
    }

    #[test]
    fn test_run_ip_returns_none_or_error_message() {
        // The `ip` command resolves the local interface via a UDP socket.
        // It succeeds (returns None) when the network stack is usable, or
        // returns Some("Error getting IP: …") when it is not. Either outcome
        // is valid; we assert the dispatch routes correctly without panicking.
        let result = run(Some(Commands::Ip));
        if let Some(msg) = &result {
            assert!(
                msg.starts_with("Error getting IP"),
                "unexpected ip dispatch result: {msg}"
            );
        }
    }

    #[test]
    fn test_run_disable_returns_none_when_available() {
        // Disable must succeed (return None) when git/npm are present, even if
        // no proxy was previously configured. This covers the Disable dispatch
        // branch and the idempotent unset path.
        let _guard = CONFIG_LOCK.lock().unwrap();
        if git_available().is_err() {
            eprintln!("Skipping: git/npm not available");
            return;
        }

        // Ensure a clean starting state.
        let _ = disable_proxy();

        let result = run(Some(Commands::Disable));
        assert!(
            result.is_none(),
            "disable should succeed (return None) when git/npm available: {result:?}"
        );
    }

    #[test]
    fn test_run_enable_returns_none_when_available() {
        // Enable must succeed (return None) on a valid proxy URL and must be
        // cleaned up afterwards so global config is left untouched. This covers
        // the Enable dispatch branch.
        let _guard = CONFIG_LOCK.lock().unwrap();
        if git_available().is_err() {
            eprintln!("Skipping: git/npm not available");
            return;
        }

        // Start from a known state, then restore it after the test.
        let _ = disable_proxy();
        let result = run(Some(Commands::Enable(test_enable_args())));
        let cleanup = disable_proxy();

        assert!(
            cleanup.is_ok(),
            "cleanup disable_proxy should succeed after enable test"
        );
        assert!(
            result.is_none(),
            "enable should succeed (return None) when git/npm available: {result:?}"
        );
    }

    #[test]
    fn test_run_pin_success_returns_none() {
        // pin to localhost should succeed and dispatch should return None.
        let args = default_ping_args("127.0.0.1");
        let result = run(Some(Commands::Pin(args)));
        assert!(
            result.is_none(),
            "pin to 127.0.0.1 should succeed (return None), got: {result:?}"
        );
    }

    #[test]
    fn test_run_pin_failure_returns_error() {
        // pin to an invalid host should fail and dispatch should return
        // Some(error_message) containing the destination.
        let args = PingArgs {
            destination: "this-host-does-not-exist.invalid".to_string(),
            ..default_ping_args("placeholder")
        };
        let result = run(Some(Commands::Pin(args)));
        assert!(result.is_some(), "pin to invalid host should error");
        let msg = result.unwrap();
        assert!(
            msg.contains("this-host-does-not-exist"),
            "error should mention the destination: {msg}"
        );
    }

    #[test]
    fn test_run_ping_success_returns_none() {
        // Ping to localhost should succeed and dispatch should return None.
        let args = default_ping_args("127.0.0.1");
        let result = run(Some(Commands::Ping(args)));
        assert!(
            result.is_none(),
            "ping to 127.0.0.1 should succeed (return None), got: {result:?}"
        );
    }

    #[test]
    fn test_run_ping_failure_returns_error() {
        // Ping to an invalid host should fail and dispatch should return
        // Some(error_message) containing the destination.
        let args = PingArgs {
            destination: "this-host-does-not-exist.invalid".to_string(),
            ..default_ping_args("placeholder")
        };
        let result = run(Some(Commands::Ping(args)));
        assert!(result.is_some(), "ping to invalid host should error");
        let msg = result.unwrap();
        assert!(
            msg.contains("this-host-does-not-exist"),
            "error should mention the destination: {msg}"
        );
    }

    #[test]
    fn test_cli_parse_enable_command() {
        let cli = Cli::try_parse_from(["proxy-x", "enable", "http://localhost:8080"]).unwrap();
        match cli.command {
            Some(Commands::Enable(args)) => {
                assert_eq!(args.proxy_url, "http://localhost:8080");
            }
            _ => panic!("expected Enable command"),
        }
    }

    #[test]
    fn test_cli_parse_disable_command() {
        let cli = Cli::try_parse_from(["proxy-x", "disable"]).unwrap();
        assert!(matches!(cli.command, Some(Commands::Disable)));
    }

    #[test]
    fn test_cli_parse_ip_command() {
        let cli = Cli::try_parse_from(["proxy-x", "ip"]).unwrap();
        assert!(matches!(cli.command, Some(Commands::Ip)));
    }

    #[test]
    fn test_cli_parse_ping_with_custom_args() {
        let cli =
            Cli::try_parse_from(["proxy-x", "ping", "8.8.8.8", "-c", "10", "-s", "128"]).unwrap();
        match cli.command {
            Some(Commands::Ping(args)) => {
                assert_eq!(args.destination, "8.8.8.8");
                assert_eq!(args.count, 10);
                assert_eq!(args.size, 128);
            }
            _ => panic!("expected Ping command"),
        }
    }

    #[test]
    fn test_cli_parse_ping_defaults() {
        let cli = Cli::try_parse_from(["proxy-x", "ping", "example.com"]).unwrap();
        match cli.command {
            Some(Commands::Ping(args)) => {
                assert_eq!(args.destination, "example.com");
                // Default values from clap
                assert_eq!(args.count, 4);
                assert_eq!(args.size, 56);
                assert_eq!(args.ttl, 64);
                assert_eq!(args.interval, 1000);
            }
            _ => panic!("expected Ping command"),
        }
    }

    #[test]
    fn test_cli_parse_pin_command() {
        let cli = Cli::try_parse_from(["proxy-x", "pin", "8.8.8.8"]).unwrap();
        assert!(matches!(cli.command, Some(Commands::Pin(_))));
    }

    #[test]
    fn test_cli_enable_requires_proxy_url() {
        // `enable` requires a proxy_url argument
        let result = Cli::try_parse_from(["proxy-x", "enable"]);
        assert!(result.is_err(), "enable without proxy_url should fail");
    }

    #[test]
    fn test_cli_ping_requires_destination() {
        // `ping` requires a destination argument
        let result = Cli::try_parse_from(["proxy-x", "ping"]);
        assert!(result.is_err(), "ping without destination should fail");
    }
}
