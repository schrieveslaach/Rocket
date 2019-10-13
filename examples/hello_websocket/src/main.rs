#![feature(proc_macro_hygiene)]

#[macro_use]
extern crate rocket;

use rocket::message::{channel, Message};

#[get("/")]
fn hello() -> &'static str {
    "Hello, websockets!"
}

fn main() {
    let (tx, rx) = channel();

    std::thread::spawn(move || {
        let duration = std::time::Duration::from_secs(1);
        loop {
            println!("Sending message");
            tx.unbounded_send(Message{}).unwrap();
            std::thread::sleep(duration);
        }
    });

    let _ = rocket::ignite().receivers(vec![rx]).mount("/", routes![hello]).launch();
}
