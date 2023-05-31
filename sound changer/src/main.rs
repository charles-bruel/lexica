#![feature(string_remove_matches)]

extern crate fancy_regex;
extern crate tungstenite;
extern crate priority_queue;
extern crate no_panic;
extern crate serde;
extern crate clap;

pub mod args;
pub mod io;
pub mod constructor;
pub mod rules;
pub mod data;
pub mod applicator;
pub mod websocket_handler;
#[cfg(test)]
mod tests;

use args::LexicaArgs;
use clap::Parser;

fn main() {
    let args = args::LexicaArgs::parse();

    match args.mode {
        args::LexicaMode::WebIO => io::web_socket_listener(),
        args::LexicaMode::Manual(command) => match command.command {
            args::ManualSubcommand::Rebuild(v) => todo!(),
        },
    }
}