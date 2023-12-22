use clap::{command, Arg, ArgMatches, Command};

use proxy_x::{disable_proxy, enable_proxy, get_agent_ip};

pub fn execute() {
    let matches = parser();
    handler(matches);
}

fn parser() -> ArgMatches {
    command!() // requires "cargo" feature in clap
        .arg_required_else_help(true)
        .subcommands(vec![
            Command::new("ip").about("Access the agent's IP address"),
            Command::new("enable")
                .about("Enables the proxy")
                .arg(Arg::new("proxy_url").required(true)),
            Command::new("disable").about("Disables the proxy"),
        ])
        .get_matches()
}

fn handler(matches: ArgMatches) {
    match matches.subcommand_name() {
        Some("enable") => {
            let sub_matches = matches.subcommand_matches("enable").unwrap();
            let proxy_url = sub_matches.get_one::<String>("proxy_url").unwrap();
            enable_proxy(proxy_url);
        }
        Some("disable") => {
            disable_proxy();
        }
        Some("ip") => match get_agent_ip() {
            Ok(ip) => println!("Current IP: {}", ip),
            Err(e) => eprintln!("Error getting IP: {}", e),
        },
        _ => {}
    }
}
