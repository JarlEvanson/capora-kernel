//! Helper crate for building and testing the capora kernel.

use std::{fmt, path::PathBuf};

use cli::{parse_arguments, Action, Arch};

pub mod cli;

fn main() {
    match parse_arguments() {
        Action::Build { arch, release } => match build(arch, release) {
            Ok(path) => println!("kernel located at \"{}\"", path.display()),
            Err(error) => {
                eprintln!("{error:?}");
            }
        },
        Action::Run { arch, release, ovmf_code, ovmf_vars } => todo!(),
    };
}

/// Builds the Capora kernel.
pub fn build(arch: Arch, release: bool) -> Result<PathBuf, BuildError> {
    let mut cmd = std::process::Command::new("cargo");
    cmd.arg("build");
    cmd.args(["--package", "kernel"]);

    cmd.args(["--target", arch.as_target_triple()]);
    if release {
        cmd.arg("--release");
    }

    let mut binary_location = PathBuf::with_capacity(50);
    binary_location.push("target");
    binary_location.push(arch.as_target_triple());
    if release {
        binary_location.push("release");
    } else {
        binary_location.push("debug");
    }
    binary_location.push("kernel");

    let status = cmd.status()?;
    if !status.success() {
        return Err(BuildError::UnsuccessfulBuild {
            code: status.code(),
        });
    }

    Ok(binary_location)
}

/// Various errors that can occur while building the Capora kernel.
#[derive(Debug)]
pub enum BuildError {
    /// An error occurred while launching the process.
    ProcessError(std::io::Error),
    /// The build was unsuccessful.
    UnsuccessfulBuild {
        /// The exit code of the child process that was launched.
        code: Option<i32>,
    },
}

impl From<std::io::Error> for BuildError {
    fn from(value: std::io::Error) -> Self {
        Self::ProcessError(value)
    }
}

impl fmt::Display for BuildError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::UnsuccessfulBuild { code: Some(code) } => writeln!(
                f,
                "error while building kernel: command failed with exit status {code}",
            ),
            Self::UnsuccessfulBuild { code: None } => {
                f.write_str("error while building kernel: command terminated by signal")
            }
            Self::ProcessError(error) => writeln!(f, "error while launching process: {error}",),
        }
    }
}
