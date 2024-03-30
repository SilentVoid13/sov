use std::path::PathBuf;

use clap::{Parser, Subcommand};
use sov_core::SovFeature;

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
    Rename {
        path: PathBuf,
        new_filename: String,
    },
    Script {
        #[command(subcommand)]
        cmd: ScriptCommand,
    },
    Search {
        #[command(subcommand)]
        cmd: SearchCommand,
    },
    Daily,
}

#[derive(Subcommand, Debug)]
pub enum ListCommand {
    Tags,
    /// Orphans are notes that are not linked to any other note
    Orphans,
    /// Dead links are notes that are linked to, but do not exist
    DeadLinks,
    Scripts,
}

#[derive(Subcommand, Debug)]
pub enum ScriptCommand {
    Run {
        script_name: String,
        args: Vec<String>,
    },
    Create {
        note_name: String,
        script_name: String,
        args: Vec<String>,
    },
}

#[derive(Subcommand, Debug)]
pub enum SearchCommand {
    Tag {
        name: String
    },
}

impl From<SovCmd> for SovFeature {
    fn from(cmd: SovCmd) -> Self {
        match cmd {
            SovCmd::Index => SovFeature::Index,
            SovCmd::List { cmd } => match cmd {
                ListCommand::Tags => SovFeature::ListTags,
                ListCommand::Orphans => SovFeature::ListOrphans,
                ListCommand::DeadLinks => SovFeature::ListDeadLinks,
                ListCommand::Scripts => SovFeature::ListScripts,
            },
            SovCmd::Resolve { note } => SovFeature::ResolveNote { note },
            SovCmd::Rename { path, new_filename } => SovFeature::Rename { path, new_filename },
            SovCmd::Daily => SovFeature::Daily,
            SovCmd::Script { cmd } => match cmd {
                ScriptCommand::Run { script_name, args } => {
                    SovFeature::ScriptRun { script_name, args }
                }
                ScriptCommand::Create {
                    note_name,
                    script_name,
                    args,
                } => SovFeature::ScriptCreate {
                    note_name,
                    script_name,
                    args,
                },
            },
            SovCmd::Search { cmd } => match cmd {
                SearchCommand::Tag { name } => SovFeature::SearchTag { tag: name },
            },
        }
    }
}
