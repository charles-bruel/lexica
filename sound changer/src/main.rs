#![feature(string_remove_matches)]

extern crate fancy_regex;
extern crate tungstenite;
extern crate priority_queue;

pub mod io;
pub mod constructor;
pub mod rules;
pub mod data;
pub mod applicator;

fn main() {
    use std::time::Instant;
    let now = Instant::now();

    let prog = constructor::construct(io::load_from_file(&String::from("sava1.csc")));

    let mut words = constructor::construct_words(&prog, io::load_from_file(&String::from("sava-words.txt")));

    words = prog.apply_vec(words);

    let elapsed = now.elapsed();
    print!("Total runtime: {:.2?}\n", elapsed);

    for w in &words {
        for l in w {
            print!("{}", l.get_symbol(&prog));
        }
        println!();
    }
}