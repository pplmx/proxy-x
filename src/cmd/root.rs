use clap::{Command, Arg, ArgAction};
use proxy_rs::proxy_manager;

pub fn execute() {
    let mut command = Command::new("Proxy Manager")
        .version("1.0")
        .author("Mystic")
        .about("Manages proxy settings for git and npm")
        .arg(
            Arg::new("enable")
                .long("enable")
                .action(ArgAction::Set) // 设置为 true 当标志被使用时
                .value_name("PROXY_URL")
                .help("Enables the proxy with the given URL")
                .num_args(1), // 接受一个参数
        )
        .arg(
            Arg::new("disable")
                .long("disable")
                .action(ArgAction::SetFalse) // 设置为 false 当标志被使用时
                .help("Disables the proxy")
                .num_args(0), // 不接受参数
        );

    let matches = command.clone().get_matches();

    if let Some(proxy_url) = matches.get_one::<String>("enable") {
        proxy_manager::enable_proxy(proxy_url);
    } else if matches.contains_id("disable") {
        proxy_manager::disable_proxy();
    } else {
        command.print_help().unwrap();
    }
}
