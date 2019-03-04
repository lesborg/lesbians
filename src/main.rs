// SPDX-License-Identifier: AGPL-3.0-only

mod db;
mod item;

use crate::db::Db;
use failure::Fallible;
use std::io;
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
}

fn main() -> Fallible<()> {
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("info")).init();

    let opt = Opt::from_args();
    let db = Db::open(opt.db_path)?;
    match opt.cmd {
        SubCommand::Dump => db.dump(io::stdout()),
        SubCommand::Restore => db.restore(io::stdin().lock()),
    }
}
