// SPDX-License-Identifier: AGPL-3.0-only

#![warn(clippy::pedantic)]
#![allow(clippy::use_self)]

mod date;
mod db;
mod format;
mod isbn;
mod item;
mod lesb;
mod location;

use crate::db::Db;
use crate::item::Item;
use failure::Fallible;
use std::io;
use std::io::prelude::*;
use std::path::PathBuf;
use structopt::StructOpt;

#[derive(Debug, StructOpt)]
struct Opt {
    #[structopt(parse(from_os_str))]
    db_path: PathBuf,
    #[structopt(subcommand)]
    cmd: SubCommand,
}

#[derive(Debug, StructOpt)]
enum SubCommand {
    #[structopt(name = "dump")]
    Dump,
    #[structopt(name = "restore")]
    Restore,
    #[structopt(name = "search")]
    Search { query: String },
}

fn main() -> Fallible<()> {
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("lesbians=info"))
        .init();

    let opt = Opt::from_args();
    let mut db = Db::open(opt.db_path)?;
    match opt.cmd {
        SubCommand::Dump => db.dump(io::stdout()),
        SubCommand::Restore => db.restore(io::stdin().lock()),
        SubCommand::Search { query } => {
            for item in db.query::<Item>(&query)? {
                serde_json::to_writer(&mut io::stdout(), &item)?;
                io::stdout().write_all(b"\n")?;
            }
            Ok(())
        }
    }
}
