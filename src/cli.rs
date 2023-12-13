use clap::{crate_authors, crate_description, crate_version, Arg, ArgMatches, Command};

use proxy_x::{disable_proxy, enable_proxy};

pub fn execute() {
    let matches = parser();
    handler(matches);
}

fn parser() -> ArgMatches {
    Command::new("Proxy Manager")
        .arg_required_else_help(true)
        .version(crate_version!())
        .author(crate_authors!("\n"))
        .about(crate_description!())
        .arg(
            Arg::new("enable")
                .long("enable")
                .short('e')
                .value_name("PROXY_URL")
                .help("Enables the proxy with the given URL")
                .num_args(1), // 接受一个参数
        )
        .arg(
            Arg::new("disable")
                .long("disable")
                .short('d')
                .help("Disables the proxy")
                .num_args(0), // 不接受参数
        )
        .get_matches()
}

fn handler(matches: ArgMatches) {
    if let Some(proxy_url) = matches.get_one::<String>("enable") {
        enable_proxy(proxy_url);
    } else if matches.contains_id("disable") {
        disable_proxy();
    }
}
