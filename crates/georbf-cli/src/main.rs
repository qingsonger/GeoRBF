//! Minimal command-line bootstrap for `GeoRBF`.

#![forbid(unsafe_code)]

use std::env;
use std::process::ExitCode;

const HELP: &str = "GeoRBF command-line interface\n\nUsage: georbf [--help | --version]\n\nStage 0 exposes no fitting commands.";

fn dispatch(mut args: impl Iterator<Item = String>) -> Result<String, String> {
    match args.next().as_deref() {
        Some("--help" | "-h") => Ok(HELP.to_owned()),
        Some("--version" | "-V") => Ok(format!("georbf {}", env!("CARGO_PKG_VERSION"))),
        Some(argument) => Err(format!(
            "unsupported argument `{argument}`; stage 0 exposes only --help and --version"
        )),
        None => Err("no command supplied; use --help for the stage-0 interface".to_owned()),
    }
}

fn main() -> ExitCode {
    match dispatch(env::args().skip(1)) {
        Ok(output) => {
            println!("{output}");
            ExitCode::SUCCESS
        }
        Err(message) => {
            eprintln!("error: {message}");
            ExitCode::from(2)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{HELP, dispatch};

    #[test]
    fn reports_workspace_version() {
        let output = dispatch(["--version".to_owned()].into_iter());
        assert_eq!(output.as_deref(), Ok("georbf 0.0.1"));
    }

    #[test]
    fn documents_that_business_commands_are_unavailable() {
        let output = dispatch(["--help".to_owned()].into_iter());
        assert_eq!(output.as_deref(), Ok(HELP));
    }

    #[test]
    fn rejects_unimplemented_commands() {
        let result = dispatch(["fit".to_owned()].into_iter());
        assert!(result.is_err());
    }
}
