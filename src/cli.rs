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
    match cli.command {
        Some(Commands::Ip) => match get_agent_ip() {
            Ok(ip) => println!("Current IP: {}", ip),
            Err(e) => eprintln!("Error getting IP: {}", e),
        },
        Some(Commands::Enable(args)) => match enable_proxy(&args.proxy_url) {
            Ok(()) => {}
            Err(e) => eprintln!("Error enabling proxy: {}", e),
        },
        Some(Commands::Disable) => match disable_proxy() {
            Ok(()) => {}
            Err(e) => eprintln!("Error disabling proxy: {}", e),
        },
        Some(Commands::Ping(args)) => match ping() {
            Ok(()) => println!("Ping to {} completed", args.destination),
            Err(e) => eprintln!("Error: {}", e),
        },
        Some(Commands::Pin(args)) => match async_ping() {
            Ok(()) => println!("Ping to {} completed", args.destination),
            Err(e) => eprintln!("Error: {}", e),
        },
        None => {}
    }
}
