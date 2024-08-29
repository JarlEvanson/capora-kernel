//! Helper crate for building and testing the capora kernel.

use std::{ffi::OsString, fmt, path::PathBuf};

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
        Action::Run {
            arch,
            release,
            ovmf_code,
            ovmf_vars,
            limine_path,
        } => match run(arch, release, ovmf_code, ovmf_vars, limine_path) {
            Ok(_) => {}
            Err(error) => {
                eprintln!("{error}");
            }
        },
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

/// Builds and runs the Capora kernel.
pub fn run(
    arch: Arch,
    release: bool,
    ovmf_code: PathBuf,
    ovmf_vars: PathBuf,
    limine_path: PathBuf,
) -> Result<(), RunError> {
    let kernel_path = build(arch, release)?;
    let fat_directory = build_fat_directory(arch, kernel_path, limine_path)
        .map_err(RunError::BuildFatDirectoryError)?;

    let qemu_name = match arch {
        Arch::X86_64 => "qemu-system-x86_64",
    };

    let mut cmd = std::process::Command::new(qemu_name);

    // Disable unnecessary devices.
    cmd.arg("-nodefaults");

    cmd.args(["-boot", "menu=on,splash-time=0"]);
    match arch {
        Arch::X86_64 => {
            // Use fairly modern machine to target.
            cmd.args(["-machine", "q35"]);

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
    ovmf_code_arg.push(ovmf_code);
    cmd.arg("-drive").arg(ovmf_code_arg);

    let mut ovmf_vars_arg = OsString::from("if=pflash,format=raw,readonly=on,file=");
    ovmf_vars_arg.push(ovmf_vars);
    cmd.arg("-drive").arg(ovmf_vars_arg);

    let mut fat_drive_arg = OsString::from("format=raw,file=fat:rw:");
    fat_drive_arg.push(fat_directory);
    cmd.arg("-drive").arg(fat_drive_arg);

    let status = cmd.status()?;
    if !status.success() {
        return Err(RunError::QemuError {
            code: status.code(),
        });
    }

    Ok(())
}

/// Various errors that can occur while building and running the Capora kernel.
#[derive(Debug)]
pub enum RunError {
    /// An error occurred while building the kernel.
    BuildError(BuildError),
    /// An error occurred while building the fat directory.
    BuildFatDirectoryError(std::io::Error),
    /// An error occurred while launching QEMU.
    ProcessError(std::io::Error),
    /// QEMU exited with a non-zero exit code.
    QemuError {
        /// The exit code of QEMU.
        code: Option<i32>,
    },
}

impl From<BuildError> for RunError {
    fn from(value: BuildError) -> Self {
        Self::BuildError(value)
    }
}

impl From<std::io::Error> for RunError {
    fn from(value: std::io::Error) -> Self {
        Self::ProcessError(value)
    }
}

impl fmt::Display for RunError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::BuildError(error) => fmt::Display::fmt(error, f),
            Self::BuildFatDirectoryError(error) => {
                writeln!(f, "error occurred while building FAT directory: {error}",)
            }
            Self::ProcessError(error) => writeln!(f, "error while launching QEMU: {error}"),
            Self::QemuError { code: Some(code) } => writeln!(
                f,
                "error while running QEMU: command failed with exit status {code}"
            ),
            Self::QemuError { code: None } => {
                writeln!(f, "error while running QEMU: command terminated by signal")
            }
        }
    }
}

const LIMINE_CONF: &str = "\
    timeout: 0\n\
    \n\
    /Capora Kernel\n\
        \tprotocol: limine\n\
        \tkernel_path: boot():/kernel
";

/// Sets up the FAT directory used for UEFI boot.
pub fn build_fat_directory(
    arch: Arch,
    kernel_path: PathBuf,
    limine_path: PathBuf,
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

    std::fs::copy(limine_path, boot_directory.join(boot_file_name))?;
    std::fs::write(fat_directory.join("limine.conf"), LIMINE_CONF)?;
    std::fs::copy(kernel_path, fat_directory.join("kernel"))?;

    Ok(fat_directory)
}
