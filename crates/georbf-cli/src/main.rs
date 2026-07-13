//! Minimal command-line bootstrap for `GeoRBF`.

#![forbid(unsafe_code)]

use std::env;
use std::ffi::OsString;
use std::process::ExitCode;

const HELP: &str = "GeoRBF command-line interface\n\nUsage: georbf [--help | --version]\n\nStage 0 exposes no fitting commands.";

fn dispatch(mut args: impl Iterator<Item = OsString>) -> Result<String, String> {
    let Some(argument) = args.next() else {
        return Err("no command supplied; use --help for the stage-0 interface".to_owned());
    };
    if args.next().is_some() {
        return Err("stage 0 accepts exactly one of --help or --version".to_owned());
    }
    let Some(argument) = argument.to_str() else {
        return Err("argument is not valid Unicode".to_owned());
    };
    match argument {
        "--help" | "-h" => Ok(HELP.to_owned()),
        "--version" | "-V" => Ok(format!("georbf {}", env!("CARGO_PKG_VERSION"))),
        argument => Err(format!(
            "unsupported argument `{argument}`; stage 0 exposes only --help and --version"
        )),
    }
}

fn main() -> ExitCode {
    match dispatch(env::args_os().skip(1)) {
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
    use std::ffi::OsString;

    use super::{HELP, dispatch};

    #[test]
    fn reports_workspace_version() {
        let output = dispatch([OsString::from("--version")].into_iter());
        assert_eq!(output.as_deref(), Ok("georbf 0.0.1"));
    }

    #[test]
    fn documents_that_business_commands_are_unavailable() {
        let output = dispatch([OsString::from("--help")].into_iter());
        assert_eq!(output.as_deref(), Ok(HELP));
    }

    #[test]
    fn rejects_unimplemented_commands() {
        let result = dispatch([OsString::from("fit")].into_iter());
        assert!(result.is_err());
    }

    #[test]
    fn rejects_extra_arguments() {
        let result = dispatch([OsString::from("--version"), OsString::from("fit")].into_iter());
        assert!(result.is_err());
    }

    #[cfg(unix)]
    #[test]
    fn rejects_non_unicode_arguments() {
        use std::os::unix::ffi::OsStringExt;

        let result = dispatch([OsString::from_vec(vec![0xff])].into_iter());
        assert!(result.is_err());
    }
}
