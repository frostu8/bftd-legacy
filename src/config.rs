//! Application configuration.

use clap::{Command, Arg};

/// Executable arguments.
pub struct Args {
    pub netmode: u32,
}

impl Args {
    /// Parses the arguments and returns this struct.
    pub fn from_args() -> Args {
        let m = Command::new(env!("CARGO_PKG_NAME"))
            .author(env!("CARGO_PKG_AUTHORS"))
            .version(env!("CARGO_PKG_VERSION"))
            .about(env!("CARGO_PKG_DESCRIPTION"))
            .arg(
                Arg::new("netmode")
                    .long("netmode")
                    .default_value("0")
            )
            .get_matches();

        Args {
            netmode: m.value_of("netmode").unwrap().parse().unwrap(),
        }
    }
}

