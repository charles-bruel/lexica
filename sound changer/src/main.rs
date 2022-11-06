#![feature(string_remove_matches)]

extern crate fancy_regex;
extern crate tungstenite;
extern crate priority_queue;
extern crate no_panic;
extern crate serde;

pub mod io;
pub mod constructor;
pub mod rules;
pub mod data;
pub mod applicator;
pub mod websocket_handler;
#[cfg(test)]
mod tests;

fn main() {
    io::web_socket_listener();
}