use std::fs::File;
use std::io::prelude::*;
use std::path::Path;
use std::net::TcpListener;
use std::thread::spawn;
use tungstenite::{accept, Message};

//https://doc.rust-lang.org/rust-by-example/std_misc/file/open.html
pub fn load_from_file(path_str: &String) -> String {
    use std::time::Instant;
    let now = Instant::now();

    let path = Path::new(path_str);
    let display = path.display();

    let mut file = match File::open(&path) {
        Err(why) => panic!("couldn't open {}: {}", display, why),
        Ok(file) => file,
    };

    let mut s = String::new();
    let temp = match file.read_to_string(&mut s) {
        Err(why) => panic!("couldn't read {}: {}", display, why),
        Ok(_) => s,
    };

    let elapsed = now.elapsed();
    print!("Loaded file in {:.2?}\n", elapsed);

    return temp;
}

pub fn web_socket_listener() {
    let server = TcpListener::bind("127.0.0.1:1200").unwrap();
    for stream in server.incoming() {
        spawn (move || {
            let websocket_result = accept(stream.unwrap());
            let mut websocket = match websocket_result {
                Ok(v) => v,
                Err(v) => {
                    println!("{}", v.to_string());
                    return;
                }
            };
            println!("Successfully opened websocket");
            loop {
                let msg = websocket.read_message().unwrap();

                println!("{}", msg);

                let msg: Message = Message::Text(String::from("foo baz bar"));
                websocket.write_message(msg).unwrap();
            }
        });
    }
}