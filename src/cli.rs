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
    Disable, // if no about is provided, #[command(about = "")] can be omitted
}

#[derive(Args)]
struct EnableArgs {
    #[arg(required = true)]
    proxy_url: String,
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
        None => {}
    }
}
