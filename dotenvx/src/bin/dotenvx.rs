extern crate clap;
extern crate dotenvx;

use std::os::unix::process::CommandExt;
use std::process::{exit, Command};

macro_rules! die {
    ($fmt:expr) => ({
        eprintln!($fmt);
        exit(1);
    });
    ($fmt:expr, $($arg:tt)*) => ({
        eprintln!($fmt, $($arg)*);
        exit(1);
    });
}

fn make_command(name: &str, args: Vec<&str>) -> Command {
    let mut command = Command::new(name);

    for arg in args {
        command.arg(arg);
    }

    return command;
}

fn main() {
    let matches = clap::Command::new("dotenvx")
        .about("Run a command using the environment in a .env file")
        .override_usage("dotenvx <COMMAND> [ARGS]...")
        .allow_external_subcommands(true)
        .arg_required_else_help(true)
        .arg(
            clap::Arg::new("FILE")
                .short('f')
                .long("file")
                .help("Use a specific .env file (defaults to .env)"),
        )
        .get_matches();

    match matches.get_one::<String>("FILE") {
        None => dotenvx::dotenv(),
        Some(file) => dotenvx::from_filename(file),
    }
    .unwrap_or_else(|e| die!("error: failed to load environment: {}", e));

    let mut command = match matches.subcommand() {
        Some((name, matches)) => {
            let args = matches
                .get_many("")
                .map(|v| v.copied().collect())
                .unwrap_or(Vec::new());

            make_command(name, args)
        }
        _ => die!("error: missing required argument <COMMAND>"),
    };

    if cfg!(target_os = "windows") {
        match command.spawn().and_then(|mut child| child.wait()) {
            Ok(status) => exit(status.code().unwrap_or(1)),
            Err(error) => die!("fatal: {}", error),
        };
    } else {
        let error = command.exec();
        die!("fatal: {}", error);
    };
}
