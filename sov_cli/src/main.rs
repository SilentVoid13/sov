mod args;

use clap::Parser;
use color_eyre::eyre::eyre;
use color_eyre::Result;
use sov::Sov;
use tracing::{info, Level};
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
        },
        SovCmd::Resolve { note } => {
            let path = sov.resolve_note(&note)?;
            dbg!(path);
        }
        _ => {}
    };

    Ok(())
}
