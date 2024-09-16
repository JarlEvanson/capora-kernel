//! Command line parsing and command construction.

use std::{
    ops::{BitAnd, BitOr},
    path::PathBuf,
};

use clap::ArgAction;

/// The action to carry out.
pub enum Action {
    /// Build the Capora kernel.
    Build(BuildArguments),
    /// Build and run the Capora kernel using Limine.
    RunLimine {
        /// Arguments necessary to build the Capora kernel.
        build_arguments: BuildArguments,
        /// Arguments necessary to run the Capora kernel.
        run_arguments: RunArguments,
        /// The path to the Limine bootloader.
        limine_path: PathBuf,
    },
    /// Build and run the Capora kernel using `capora-boot-stub`.
    RunBootStub {
        /// Arguments necessary to build the Capora kernel.
        build_arguments: BuildArguments,
        /// Argument necessary to run the Capora kernel.
        run_arguments: RunArguments,
    },
}

/// Arguments necessary to determine how to build the kernel.
#[derive(Clone, Copy, Debug, Hash, PartialEq, Eq)]
pub struct BuildArguments {
    /// THe architecture for which the kernel should be built.
    pub arch: Arch,
    /// Whether the kernel should be built in release mode.
    pub release: bool,
    /// The features that the kernel should have enabled.
    pub features: Features,
}

/// Arguments necessary to determine how to run the kernel.
pub struct RunArguments {
    /// The path to the OVMF code file used to run UEFI.
    pub ovmf_code: PathBuf,
    /// The path to the OVMF vars file used to run UEFI.
    pub ovmf_vars: PathBuf,
}

/// Parses arguments to construct an [`Action`].
pub fn parse_arguments() -> Action {
    let mut matches = command_parser().get_matches();
    let (subcommand_name, mut subcommand_matches) =
        matches.remove_subcommand().expect("subcommand required");
    match subcommand_name.as_str() {
        "build" => Action::Build(parse_build_arguments(&mut subcommand_matches)),
        "run-limine" => Action::RunLimine {
            build_arguments: parse_build_arguments(&mut subcommand_matches),
            run_arguments: parse_run_arguments(&mut subcommand_matches),
            limine_path: subcommand_matches
                .remove_one("limine")
                .expect("limine is required"),
        },
        "run-boot-stub" => Action::RunBootStub {
            build_arguments: parse_build_arguments(&mut subcommand_matches),
            run_arguments: parse_run_arguments(&mut subcommand_matches),
        },
        name => unreachable!("unexpected subcommand {name:?}"),
    }
}

/// Parses subcommand arguments for the [`Action::Build`] subcommand.
pub fn parse_build_arguments(matches: &mut clap::ArgMatches) -> BuildArguments {
    let arch = matches
        .remove_one::<Arch>("arch")
        .expect("arch is a required argument");
    let release = matches.remove_one::<bool>("release").unwrap_or(false);

    let mut features = Features::default();
    for feature in matches
        .get_many::<String>("features")
        .into_iter()
        .flatten()
        .map(String::as_str)
        .flat_map(|s| parse_feature(&s))
    {
        let new_feature = match feature {
            "limine-boot-api" => Features::LIMINE_BOOT_API,
            "capora-boot-api" => Features::CAPORA_BOOT_API,
            "debugcon" => Features::DEBUGCON,
            feature => {
                eprintln!("unsupported feature `{feature}`");
                std::process::exit(1);
            }
        };

        features = features | new_feature;
    }

    BuildArguments {
        arch,
        release,
        features,
    }
}

fn parse_feature<'str>(feature: &'str str) -> impl Iterator<Item = &'str str> + 'str {
    feature
        .split_whitespace()
        .flat_map(|s| s.split(','))
        .filter(|s| !s.is_empty())
}

/// Parses subcommand arguments for the [`Action::Run`] subcommand.
pub fn parse_run_arguments(matches: &mut clap::ArgMatches) -> RunArguments {
    let ovmf_code = matches
        .remove_one("ovmf-code")
        .expect("ovmf-code is required");
    let ovmf_vars = matches
        .remove_one("ovmf-vars")
        .expect("ovmf-vars is required");

    RunArguments {
        ovmf_code,
        ovmf_vars,
    }
}

/// Returns the clap command parser.
pub fn command_parser() -> clap::Command {
    let arch_arg = clap::Arg::new("arch")
        .long("arch")
        .value_parser(clap::builder::EnumValueParser::<Arch>::new())
        .required(true);

    let release_arg = clap::Arg::new("release")
        .help("build the Capora kernel in release mode")
        .long("release")
        .short('r')
        .action(clap::ArgAction::SetTrue);

    let features_arg = clap::Arg::new("features")
        .help("List of features to activate")
        .long("features")
        .short('F')
        .action(ArgAction::Append);

    let build_subcommand = clap::Command::new("build")
        .about("build the Capora kernel")
        .arg(
            arch_arg
                .clone()
                .help("The architecture for which the kernel should be built"),
        )
        .arg(release_arg.clone())
        .arg(features_arg.clone());

    let ovmf_code_arg = clap::Arg::new("ovmf-code")
        .long("ovmf-code")
        .short('c')
        .value_parser(clap::builder::PathBufValueParser::new())
        .required(true);

    let ovmf_vars_arg = clap::Arg::new("ovmf-vars")
        .long("ovmf-vars")
        .short('v')
        .value_parser(clap::builder::PathBufValueParser::new())
        .required(true);

    let run_limine_subcommand = clap::Command::new("run-limine")
        .about("Run the Capora kernel using the Limine bootloader")
        .arg(
            arch_arg
                .clone()
                .help("The architecture for which the kernel should be built and run"),
        )
        .arg(release_arg.clone())
        .arg(features_arg.clone())
        .arg(ovmf_code_arg.clone())
        .arg(ovmf_vars_arg.clone())
        .arg(
            clap::Arg::new("limine")
                .long("limine")
                .short('l')
                .value_parser(clap::builder::PathBufValueParser::new())
                .required(true),
        );

    let run_boot_stub_subcommand = clap::Command::new("run-boot-stub")
        .about("Run the capora-kernel using `capora boot stub`")
        .arg(arch_arg.help("The architecture for which the kernel should be built and run"))
        .arg(release_arg)
        .arg(features_arg)
        .arg(ovmf_code_arg)
        .arg(ovmf_vars_arg);

    clap::Command::new("xtask")
        .about("Developer utility for running various tasks in capora-kernel")
        .subcommand(build_subcommand)
        .subcommand(run_limine_subcommand)
        .subcommand(run_boot_stub_subcommand)
        .subcommand_required(true)
        .arg_required_else_help(true)
}

/// The architectures supported by the kernel.
#[derive(Clone, Copy, Debug, Hash, PartialEq, Eq)]
pub enum Arch {
    /// The `x86_64` architecture.
    X86_64,
}

impl Arch {
    /// Returns the [`Arch`] as its rustc target triple.
    pub fn as_target_triple(&self) -> &'static str {
        match self {
            Self::X86_64 => "x86_64-unknown-none",
        }
    }

    /// Returns the [`Arch`] as its textual representation.
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::X86_64 => "x86_64",
        }
    }
}

impl clap::ValueEnum for Arch {
    fn value_variants<'a>() -> &'a [Self] {
        static ARCHES: &[Arch] = &[Arch::X86_64];

        ARCHES
    }

    fn to_possible_value(&self) -> Option<clap::builder::PossibleValue> {
        Some(clap::builder::PossibleValue::new(self.as_str()))
    }
}

/// The various features that should be enabled by the kernel.
#[derive(Clone, Copy, Debug, Default, Hash, PartialEq, Eq)]
pub struct Features(u64);

impl Features {
    /// Enable the `limine-boot-api` feature, which enables support for booting via the Limine boot
    /// protocol.
    pub const LIMINE_BOOT_API: Self = Self(0x1);
    /// Enables the `capora-boot-api` feature, which enables support for booting via the
    /// `capora-boot-api` protocol.
    pub const CAPORA_BOOT_API: Self = Self(0x2);
    
    /// Enables the `debugcon` feature, which enables support for using the `debugcon` device in
    /// the kernel.
    pub const DEBUGCON: Self = Self(0x4);
}

impl Features {
    /// Converts [`Features`] into a comma seperated string of the features.
    pub fn as_string(&self) -> String {
        let features = *self;
        let features = ["limine-boot-api", "capora-boot-api", "debugcon"]
            .into_iter()
            .filter(move |&f| {
                !(f == "limine-boot-api"
                    && features & Features::LIMINE_BOOT_API != Features::LIMINE_BOOT_API)
            })
            .filter(move |&f| {
                !(f == "capora-boot-api"
                    && features & Features::CAPORA_BOOT_API != Features::CAPORA_BOOT_API)
            }).filter(move |&f| {
                !(f == "debugcon" && features & Features::DEBUGCON != Features::DEBUGCON)
            });

        features.collect::<Vec<_>>().join(",")
    }
}

impl BitOr for Features {
    type Output = Self;

    fn bitor(self, rhs: Self) -> Self::Output {
        Self(self.0 | rhs.0)
    }
}

impl BitAnd for Features {
    type Output = Self;

    fn bitand(self, rhs: Self) -> Self::Output {
        Self(self.0 & rhs.0)
    }
}
