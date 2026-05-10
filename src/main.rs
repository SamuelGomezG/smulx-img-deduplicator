mod cli;
mod hasher;

use clap::Parser;

fn main() {
    let _cli = cli::Cli::parse();
    eprintln!("smulx-dedup ready");
}
