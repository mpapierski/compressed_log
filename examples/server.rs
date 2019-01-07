use ::actix::*;
use actix_web::{server, ws, App};
use lz4::Decoder;
use std::io;

/// Define http actor
struct Ws;

impl Default for Ws {
    fn default() -> Self {
        Self {}
    }
}

impl Actor for Ws {
    type Context = ws::WebsocketContext<Self>;
}

/// Handler for ws::Message message
impl StreamHandler<ws::Message, ws::ProtocolError> for Ws {
    fn handle(&mut self, msg: ws::Message, ctx: &mut Self::Context) {
        match msg {
            ws::Message::Ping(msg) => {
                println!("Ping");
                ctx.pong(&msg)
            }
            ws::Message::Text(text) => println!("Text: {} bytes", text.len()),
            ws::Message::Binary(mut bin) => {
                println!("Binary: {} bytes", bin.len());
                let bytes = bin.take().to_vec();
                let mut decoder = Decoder::new(&bytes[..]).expect("Unable to create decoder");
                let mut output: Vec<u8> = Vec::new();
                io::copy(&mut decoder, &mut output)
                    .expect("Unable to copy data from decoder to output buffer");
                println!("{}", String::from_utf8(output).unwrap());
            }
            _ => {
                println!("Unknown message");
            }
        }
    }
}

fn main() {
    println!("Start");
    server::new(|| {
        App::new()
            .resource("/sink/", |r| {
                r.f(|req| {
                    println!("Something happened!");
                    ws::start(req, Ws::default())
                })
            })
            .finish()
    })
    .bind("0.0.0.0:8000")
    .expect("Can not bind to port 8000")
    .run();
}
