//! Command line parsing and command construction.

/// The action to carry out.
pub enum Action {
    /// Build the Capora kernel.
    Build {
        /// The architecture for which the kernel should be built.
        arch: Arch,
        /// Whether the Capora kernel should be built in release mode.
        release: bool,
    },
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

/// Parses arguments to construct an [`Action`].
pub fn parse_arguments() -> Action {
    let mut matches = command_parser().get_matches();
    let (subcommand_name, subcommand_matches) =
        matches.remove_subcommand().expect("subcommand required");
    match subcommand_name.as_str() {
        "build" => parse_build_arguments(subcommand_matches),
        name => unreachable!("unexpected subcommand {name:?}"),
    }
}

/// Parses subcommand arguments for the [`Action::Build`] subcommand.
pub fn parse_build_arguments(mut matches: clap::ArgMatches) -> Action {
    let arch = matches
        .remove_one::<Arch>("arch")
        .expect("arch is a required argument");
    let release = matches.remove_one::<bool>("release").unwrap_or(false);

    Action::Build { arch, release }
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

    let build_subcommand = clap::Command::new("build")
        .about("build the Capora kernel")
        .arg(arch_arg.help("The architecture for which the kernel should be built"))
        .arg(release_arg);

    clap::Command::new("xtask")
        .about("Developer utility for running various tasks in capora-kernel")
        .subcommand(build_subcommand)
        .subcommand_required(true)
        .arg_required_else_help(true)
}
