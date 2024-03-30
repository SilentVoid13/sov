mod args;

use args::ScriptCommand;
use clap::Parser;
use color_eyre::eyre::eyre;
use color_eyre::Result;
use sov_core::Sov;
use tracing::Level;
use tracing_subscriber::prelude::*;

use crate::args::{ListCommand, SovArgs, SovCmd};

pub fn main() -> Result<()> {
    color_eyre::install()?;
    let args = SovArgs::parse();

    // Setup logging
    tracing_subscriber::fmt()
        .with_max_level(Level::INFO)
        .with_target(true)
        .finish()
        .init();

    // TODO: Use the feature instead?
    //let sov_feature = args.cmd.into();

    let mut sov = Sov::new()?;
    match args.cmd {
        SovCmd::Index => sov.index()?,
        SovCmd::List { cmd } => match cmd {
            ListCommand::Tags => {
                let tags = sov.list_tags()?;
                for tag in tags {
                    println!("{}", tag);
                }
            }
            ListCommand::Orphans => {
                let orphans = sov.list_orphans()?;
                for orphan in orphans {
                    println!("{}", orphan.to_str().ok_or(eyre!("path error"))?);
                }
            }
            ListCommand::DeadLinks => {
                let res = sov.list_dead_links()?;
                for (path, dead_link) in res {
                    println!(
                        "{}: {}",
                        path.to_str().ok_or(eyre!("path error"))?,
                        dead_link
                    );
                }
            }
            ListCommand::Scripts => {
                let scripts = sov.list_scripts()?;
                for script in scripts {
                    println!("{}", script);
                }
            }
        },
        SovCmd::Resolve { note } => {
            let path = sov.resolve_note(&note)?;
            dbg!(path);
        }
        SovCmd::Daily => {
            let note = sov.daily()?;
            dbg!(note);
        }
        SovCmd::Script { cmd } => match cmd {
            ScriptCommand::Run { script_name, args } => {
                let res = sov.script_run(&script_name, args)?;
                println!("{}", res);
            }
            ScriptCommand::Create { note_name, script_name, args } => {
                let note_path = sov.script_create(&note_name, &script_name, args)?;
                println!("{:?}", note_path);
            }
        },
        _ => {}
    };

    Ok(())
}
