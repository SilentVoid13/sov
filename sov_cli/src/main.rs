mod args;

use args::{ListCommand, SovArgs, SovCmd};

use sov::Sov;

use clap::Parser;
use color_eyre::Result;

pub fn main() -> Result<()> {
    color_eyre::install()?;
    let args = SovArgs::parse();

    let sov = Sov::new()?;
    match args.cmd {
        SovCmd::Index => sov.index()?,
        SovCmd::List { cmd } => match cmd {
            ListCommand::Tags => {
                let tags = sov.list_tags()?;
                for tag in tags {
                    println!("{}", tag);
                }
            },
            //ListCommand::Orphans => sov.list_orphans()?,
            _ => {}
        },
        _ => {}
    };

    Ok(())
}
