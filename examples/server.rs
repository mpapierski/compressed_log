use ::actix::*;
use actix_web::{server, ws, App};

/// Define http actor
struct Ws;

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
            ws::Message::Text(text) => {
                println!("Text: {} bytes", text.len());
                ()
            }
            //ctx.text(text),
            ws::Message::Binary(bin) => {
                println!("Binary: {} bytes", bin.len());
                // ctx.binary(bin),
                ()
            }
            _ => {
                println!("Unknown message");
                ()
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
                    ws::start(req, Ws)
                })
            })
            .finish()
    })
    .bind("127.0.0.1:8000")
    .expect("Can not bind to port 8000")
    .run();
}
