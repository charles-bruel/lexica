#![feature(string_remove_matches)]

use std::time::Instant;

use manual_ux::project::load_project;

extern crate clap;
extern crate fancy_regex;
extern crate no_panic;
extern crate priority_queue;
extern crate serde;
extern crate tabled;
extern crate tungstenite;

pub mod args;
pub mod io;
pub mod manual_ux;
pub mod sc;
pub mod websocket_handler;

fn main() {
    let start = Instant::now();

    use clap::Parser;

    let args = args::LexicaArgs::parse();

    match args.mode {
        args::LexicaMode::WebIO => io::web_socket_listener(),
        args::LexicaMode::Manual(command) => match command.command {
            args::ManualSubcommand::Rebuild(v) => manual_ux::rebuilder::rebuild(
                &mut load_project(command.path.clone()).unwrap(),
                v.start,
                command.path,
            ),
        },
    }

    let elapsed = start.elapsed();
    println!("Total runtime: {:?}", elapsed)
}
