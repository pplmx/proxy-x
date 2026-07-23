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
    #[command(about = "Send ICMP ECHO_REQUEST to network hosts, using pnet.")]
    Ping(PingArgs),
    #[command(about = "Send ICMP ECHO_REQUEST to network hosts, using tokio.")]
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

pub fn execute() {
    let cli = Cli::parse();
    if let Some(msg) = run(cli.command) {
        eprintln!("{msg}");
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

    #[test]
    fn test_run_none_returns_none() {
        // Running without a subcommand is a no-op — no error message.
        assert!(run(None).is_none());
    }

    #[test]
    fn test_run_pin_returns_error_message_with_tokio() {
        // `pin` is a placeholder that returns `Unsupported`. The dispatch
        // should return a `Some(error_message)` mentioning the tokio backend,
        // not just swallow the error.
        let args = default_ping_args("example.com");
        let result = run(Some(Commands::Pin(args)));
        assert!(result.is_some(), "expected an error message for pin stub");
        let msg = result.unwrap();
        assert!(
            msg.contains("tokio"),
            "error message should mention the tokio backend, got: {msg}"
        );
    }

    #[test]
    fn test_run_ping_returns_error_message_with_pnet() {
        // `ping` is a placeholder that returns `Unsupported`. The dispatch
        // should return a `Some(error_message)` mentioning the pnet backend.
        let args = default_ping_args("example.com");
        let result = run(Some(Commands::Ping(args)));
        assert!(result.is_some(), "expected an error message for ping stub");
        let msg = result.unwrap();
        assert!(
            msg.contains("pnet"),
            "error message should mention the pnet backend, got: {msg}"
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
