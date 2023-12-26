use clap::{arg, command, Args, Parser, Subcommand};

use proxy_x::{disable_proxy, enable_proxy, get_agent_ip};

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
    Disable,
    // if no about is provided, #[command(about = "")] can be omitted
    #[command(about = "Send ICMP ECHO_REQUEST to network hosts")]
    Ping(PingArgs),
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
        Some(Commands::Enable(args)) => enable_proxy(&args.proxy_url),
        Some(Commands::Disable) => disable_proxy(),
        Some(Commands::Ping(args)) => {
            println!("destination: {}", args.destination);
            println!("count: {}", args.count);
            println!("size: {}", args.size);
            println!("ttl: {}", args.ttl);
            println!("interval: {}", args.interval);
        }
        None => {}
    }
}
