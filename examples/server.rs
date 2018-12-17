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
            ws::Message::Ping(msg) => ctx.pong(&msg),
            ws::Message::Text(text) => ctx.text(text),
            ws::Message::Binary(bin) => ctx.binary(bin),
            _ => (),
        }
    }
}

fn main() {
    println!("Start");
    server::new(|| {
        App::new()
            .resource("/logs/", |r| r.f(|req| ws::start(req, Ws)))
            .finish()
    })
    .bind("127.0.0.1:8000")
    .expect("Can not bind to port 8000")
    .run();
}
