#![feature(proc_macro_hygiene)]

#[macro_use]
extern crate rocket;

use rocket::router::Route;
use rocket::handler::WebSocketHandler;
use std::pin::Pin;
use std::future::Future;

#[get("/")]
fn hello() -> &'static str {
    "Hello, websockets!"
}

//#[websocket("/echo")]
//fn echo() {
//
//}

#[derive(Clone)]
struct X {

}

impl WebSocketHandler for X {
    fn handle_upgrade(&self) -> Pin<Box<dyn Future<Output=()> + Send + 'static>> {
        unimplemented!()
    }
}

fn main() {

    let websocket = Route::websocket("/echo", X {});

    // TODO Route::websocket("/echo", );
    let _ = rocket::ignite().mount("/", routes![hello]).mount("/", vec![websocket]).launch();
}
