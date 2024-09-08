//! Helper crate for building and testing the capora kernel.

use std::{
    ffi::OsString,
    fmt, io,
    path::{Path, PathBuf},
};

use cli::{parse_arguments, Action, Arch, BuildArguments, RunArguments};

pub mod cli;

fn main() {
    match parse_arguments() {
        Action::Build(args) => match build(args) {
            Ok(path) => println!("kernel located at \"{}\"", path.display()),
            Err(error) => {
                eprintln!("{error:?}");
            }
        },
        Action::RunLimine {
            build_arguments,
            run_arguments,
            limine_path,
        } => match run_limine(build_arguments, run_arguments, limine_path) {
            Ok(_) => {}
            Err(error) => {
                eprintln!("{error}");
            }
        },
        Action::RunBootStub {
            build_arguments,
            run_arguments,
        } => match run_boot_stub(build_arguments, run_arguments) {
            Ok(_) => {}
            Err(error) => {
                eprintln!("{error}");
            }
        },
    };
}

/// Builds the Capora kernel.
pub fn build(arguments: BuildArguments) -> Result<PathBuf, BuildError> {
    let mut cmd = std::process::Command::new("cargo");
    cmd.arg("build");
    cmd.args(["--package", "kernel"]);

    cmd.args(["--target", arguments.arch.as_target_triple()]);
    if arguments.release {
        cmd.arg("--release");
    }

    let features = arguments.features.as_string();
    if features.len() != 0 {
        cmd.arg("--features").arg(features);
    }

    let mut binary_location = PathBuf::with_capacity(50);
    binary_location.push("target");
    binary_location.push(arguments.arch.as_target_triple());
    if arguments.release {
        binary_location.push("release");
    } else {
        binary_location.push("debug");
    }
    binary_location.push("kernel");

    run_cmd(cmd)?;

    Ok(binary_location)
}

/// Various errors that can occur while building the Capora kernel.
#[derive(Debug)]
pub struct BuildError(RunCommandError);

impl From<RunCommandError> for BuildError {
    fn from(value: RunCommandError) -> Self {
        Self(value)
    }
}

impl fmt::Display for BuildError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "error while building kernel: {}", self.0)
    }
}

/// Builds and runs the Capora kernel using the Limine bootloader.
pub fn run_limine(
    build_args: BuildArguments,
    run_args: RunArguments,
    limine_path: PathBuf,
) -> Result<(), RunLimineError> {
    const LIMINE_CONF: &str = "\
        timeout: 0\n\
        \n\
        /Capora Kernel\n\
            \tprotocol: limine\n\
            \tkernel_path: boot():/kernel
    ";

    let kernel_path = build(build_args)?;
    let fat_directory = build_fat_directory(
        build_args.arch,
        limine_path,
        &[(&kernel_path, "kernel")],
        &[(LIMINE_CONF.as_bytes(), "limine.conf")],
    )
    .map_err(RunLimineError::BuildFatDirectoryError)?;

    run(build_args, run_args, fat_directory)?;

    Ok(())
}

/// Various errors that can occur while building and running the Capora kernel using the Limine
/// bootloader.
#[derive(Debug)]
pub enum RunLimineError {
    /// An error occurred while building the kernel.
    BuildError(BuildError),
    /// An error occurred while building the fat directory.
    BuildFatDirectoryError(std::io::Error),
    /// An error occurred while running QEMU.
    QemuError(QemuError),
}

impl From<BuildError> for RunLimineError {
    fn from(value: BuildError) -> Self {
        Self::BuildError(value)
    }
}

impl From<QemuError> for RunLimineError {
    fn from(value: QemuError) -> Self {
        Self::QemuError(value)
    }
}

impl fmt::Display for RunLimineError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::BuildError(error) => fmt::Display::fmt(error, f),
            Self::BuildFatDirectoryError(error) => {
                writeln!(f, "error occurred while building FAT directory: {error}",)
            }
            Self::QemuError(error) => fmt::Display::fmt(error, f),
        }
    }
}

/// Builds and runs the Capora kernel using `capora-boot-stub`.
pub fn run_boot_stub(
    build_args: BuildArguments,
    run_args: RunArguments,
) -> Result<(), RunBootStubError> {
    let kernel_path = build(build_args)?;
    let fat_directory = build_fat_directory(
        build_args.arch,
        PathBuf::from(env!("CARGO_BIN_FILE_BOOT_STUB_boot-stub")),
        &[],
        &[],
    )
    .map_err(RunBootStubError::BuildFatDirectoryError)?;

    let mut cmd = std::process::Command::new(env!("CARGO_BIN_FILE_CONFIG_capora-boot-stub-ctl"));
    cmd.arg("configure");

    cmd.arg("--stub")
        .arg(fat_directory.join("EFI").join("BOOT").join("BOOTX64.EFI"));
    cmd.arg("--application")
        .arg(format!("kernel:embedded:{}", kernel_path.display()));

    run_cmd(cmd)?;

    run(build_args, run_args, fat_directory)?;

    Ok(())
}

/// Various errors that can occur while building and running the Capora kernel using
/// `capora-boot-stub`.
pub enum RunBootStubError {
    /// An error ocurred while building the kernel.
    BuildError(BuildError),
    /// An error occurred while building the fat directory.
    BuildFatDirectoryError(std::io::Error),
    /// An error occurred while configuring `capora-boot-stub`.
    ConfigureError(RunCommandError),
    /// An error occurred while running QEMU.
    QemuError(QemuError),
}

impl From<BuildError> for RunBootStubError {
    fn from(value: BuildError) -> Self {
        Self::BuildError(value)
    }
}

impl From<RunCommandError> for RunBootStubError {
    fn from(value: RunCommandError) -> Self {
        Self::ConfigureError(value)
    }
}

impl From<QemuError> for RunBootStubError {
    fn from(value: QemuError) -> Self {
        Self::QemuError(value)
    }
}

impl fmt::Display for RunBootStubError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::BuildError(error) => fmt::Display::fmt(error, f),
            Self::BuildFatDirectoryError(error) => {
                write!(f, "error occurred while building FAT directory: {error}",)
            }
            Self::ConfigureError(error) => write!(
                f,
                "error occurred while configuring `capora-boot-stub`: {error}"
            ),
            Self::QemuError(error) => fmt::Display::fmt(error, f),
        }
    }
}

/// Builds and runs the Capora kernel.
pub fn run(
    build_args: BuildArguments,
    run_args: RunArguments,
    fat_directory: PathBuf,
) -> Result<(), QemuError> {
    let qemu_name = match build_args.arch {
        Arch::X86_64 => "qemu-system-x86_64",
    };

    let mut cmd = std::process::Command::new(qemu_name);

    // Disable unnecessary devices.
    cmd.arg("-nodefaults");

    cmd.args(["-boot", "menu=on,splash-time=0"]);
    match build_args.arch {
        Arch::X86_64 => {
            // Use fairly modern machine to target.
            cmd.args(["-machine", "q35"]);
            cmd.args(["-cpu", "host,rdrand=on"]);

            // Allocate some memory.
            cmd.args(["-m", "256M"]);

            // Use vga graphics
            cmd.args(["-vga", "std"]);

            if std::env::consts::OS == "linux" {
                cmd.arg("-enable-kvm");
            }
        }
    }

    let mut ovmf_code_arg = OsString::from("if=pflash,format=raw,readonly=on,file=");
    ovmf_code_arg.push(run_args.ovmf_code);
    cmd.arg("-drive").arg(ovmf_code_arg);

    let mut ovmf_vars_arg = OsString::from("if=pflash,format=raw,readonly=on,file=");
    ovmf_vars_arg.push(run_args.ovmf_vars);
    cmd.arg("-drive").arg(ovmf_vars_arg);

    let mut fat_drive_arg = OsString::from("format=raw,file=fat:rw:");
    fat_drive_arg.push(fat_directory);
    cmd.arg("-drive").arg(fat_drive_arg);

    run_cmd(cmd)?;

    Ok(())
}

/// Various errors that can occur while running QEMU.
#[derive(Debug)]
pub struct QemuError(RunCommandError);

impl From<RunCommandError> for QemuError {
    fn from(value: RunCommandError) -> Self {
        Self(value)
    }
}

impl fmt::Display for QemuError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "error while running QEMU: {}", self.0)
    }
}

/// Sets up the FAT directory used for UEFI boot.
pub fn build_fat_directory(
    arch: Arch,
    loader_path: PathBuf,
    additional_files: &[(&Path, &str)],
    additional_binary_files: &[(&[u8], &str)],
) -> Result<PathBuf, std::io::Error> {
    let mut fat_directory = PathBuf::with_capacity(50);
    fat_directory.push("run");
    fat_directory.push(arch.as_str());
    fat_directory.push("fat_directory");

    let mut boot_directory = fat_directory.join("EFI");
    boot_directory.push("BOOT");
    if !boot_directory.exists() {
        std::fs::create_dir_all(&boot_directory)?;
    }

    let boot_file_name = match arch {
        Arch::X86_64 => "BOOTX64.EFI",
    };

    std::fs::copy(loader_path, boot_directory.join(boot_file_name))?;

    for &(file, name) in additional_files {
        std::fs::copy(file, fat_directory.join(name))?;
    }

    for &(bytes, name) in additional_binary_files {
        std::fs::write(fat_directory.join(name), bytes)?;
    }

    Ok(fat_directory)
}

/// Runs a [`Command`][c], handling non-zero exit codes and other failures.
///
/// [c]: std::process::Command
pub fn run_cmd(mut cmd: std::process::Command) -> Result<(), RunCommandError> {
    println!("Running command: {cmd:?}");

    let status = cmd.status()?;
    if !status.success() {
        return Err(RunCommandError::CommandFailed {
            code: status.code(),
        });
    }

    Ok(())
}

/// Various errors that can occur while running a command.
#[derive(Debug)]
pub enum RunCommandError {
    /// An error occurred while launching the command.
    ProcessError(io::Error),
    /// The command exited with a non-zero exit code.
    CommandFailed {
        /// The exit of code of the command.
        code: Option<i32>,
    },
}

impl From<io::Error> for RunCommandError {
    fn from(value: io::Error) -> Self {
        Self::ProcessError(value)
    }
}

impl fmt::Display for RunCommandError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::ProcessError(error) => write!(f, "error launching command: {error}"),
            Self::CommandFailed { code: Some(code) } => {
                write!(f, "command failed with exit status {code}")
            }
            Self::CommandFailed { code: None } => write!(f, "command terminated by signal"),
        }
    }
}
