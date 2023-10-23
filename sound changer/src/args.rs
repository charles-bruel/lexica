use super::clap::{Args, Parser, Subcommand};

#[derive(Debug, Parser)]
#[clap(author, version, about)]
pub struct LexicaArgs {
    #[clap(subcommand)]
    pub mode: LexicaMode,
}

#[derive(Debug, Subcommand)]
pub enum LexicaMode {
    /// Use the (WIP) web portal
    WebIO,
    /// Use the manual mode
    Manual(ManualCommand),
}

#[derive(Debug, Args)]
pub struct ManualCommand {
    /// The path to the project
    pub path: String,

    #[clap(subcommand)]
    pub command: ManualSubcommand,
}

#[derive(Debug, Subcommand)]
pub enum ManualSubcommand {
    Rebuild(ManualRebuild)
}

#[derive(Debug, Args)]
pub struct ManualRebuild {
    /// The table ID to start at
    #[arg(short, long, default_value_t = 0)]
    pub start: u16,
}