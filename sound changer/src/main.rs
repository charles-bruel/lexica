#![feature(string_remove_matches)]

use manual_ux::project::load_project;

extern crate fancy_regex;
extern crate tungstenite;
extern crate priority_queue;
extern crate no_panic;
extern crate serde;
extern crate clap;

pub mod manual_ux;
pub mod args;
pub mod io;
pub mod constructor;
pub mod rules;
pub mod data;
pub mod applicator;
pub mod websocket_handler;
#[cfg(test)]
mod tests;

fn main() {
    use clap::Parser;

    let args = args::LexicaArgs::parse();

    match args.mode {
        args::LexicaMode::WebIO => io::web_socket_listener(),
        args::LexicaMode::Manual(command) => match command.command {
            args::ManualSubcommand::Rebuild(v) => 
                manual_ux::rebuilder::rebuild(&mut load_project(command.path), v.start),
        },
    }
}