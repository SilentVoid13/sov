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
    Resolve {
        note: String,
    },
    Edit,
    Search,
}

#[derive(Subcommand, Debug)]
pub enum ListCommand {
    Tags,
    /// Orphans are notes that are not linked to any other note
    Orphans,
    /// Dead links are notes that are linked to, but do not exist
    DeadLinks
}
