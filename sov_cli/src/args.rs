use clap::{Parser, Subcommand};

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
pub struct SovArgs {
    #[command(subcommand)]
    pub cmd: SovCmd,
}

#[derive(Subcommand, Debug)]
pub enum SovCmd {
    Index,
    List {
        #[command(subcommand)]
        cmd: ListCommand,
    },
    Edit,
    Search,
}

#[derive(Subcommand, Debug)]
pub enum ListCommand {
    Tags,
    Orphans,
}
