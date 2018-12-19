use actix::{
    Actor, ActorContext, ActorFuture, Addr, Arbiter, AsyncContext, Context, ContextFutureSpawner,
    Handler, ResponseFuture, StreamHandler, Supervised, Supervisor, System, WrapFuture,
};
use actix_web::ws::{Client, ClientWriter, Message, ProtocolError};
use backoff::{backoff::Backoff, ExponentialBackoff};
use failure::Error;
use futures::future::ok;
use futures::prelude::*;
use futures::sync::mpsc;
use std::cell::RefCell;
use std::sync::{Arc, Mutex};
use std::time::{Duration, SystemTime};

// This is the websocket client
pub struct LogClient {
    url: String,
    backoff: ExponentialBackoff,
    writer: Option<ClientWriter>,
    // Time when a ping was sent
    heartbeat: SystemTime,
}

impl LogClient {
    pub fn new(url: &str) -> LogClient {
        LogClient {
            url: url.to_string(),
            backoff: ExponentialBackoff::default(),
            writer: None,
            heartbeat: SystemTime::now(),
        }
    }
}

impl LogClient {
    /// Spawns async connection to a remote websocket server,
    /// and returns a handle to a sender which will accept any
    /// kind of packet data.
    ///
    pub fn connect(url: &str) -> Addr<LogClient> {
        // Bound of 1 to limit the rate of messages
        let log_client = LogClient::new(url);
        let addr = Supervisor::start(|ctx| log_client);
        addr
    }
}

impl actix::Supervised for LogClient {
    fn restarting(&mut self, ctx: &mut Context<LogClient>) {
        println!("restarting");
    }
}

impl Actor for LogClient {
    type Context = Context<Self>;

    fn started(&mut self, ctx: &mut Context<Self>) {
        Client::new(self.url.clone())
            .connect()
            .into_actor(self)
            .map(move |(reader, writer), mut act, mut ctx| {
                // Reset the backoff timer
                act.backoff.reset();
                // Add reader stream
                LogClient::add_stream(reader, ctx);
                // Keep the writer for later
                act.writer = Some(writer);
                // Start pinging
                act.heartbeat = SystemTime::now();
                act.hb(&mut ctx);
            })
            .map_err(|err, act, ctx| {
                eprintln!("Unable to connect: {}", err);
                if let Some(timeout) = act.backoff.next_backoff() {
                    eprintln!("Next timeout: {:?}", timeout);
                    ctx.run_later(timeout, |_, ctx| ctx.stop());
                }
            })
            .wait(ctx);
        println!("Started");
    }
}

impl LogClient {
    fn hb(&self, ctx: &mut Context<Self>) {
        ctx.run_later(Duration::new(1, 0), |mut act, ctx| {
            eprintln!("Heartbeat elapsed: {:?}", act.heartbeat.elapsed().unwrap());
            if act.heartbeat.elapsed().unwrap() >= Duration::from_secs(5) {
                eprintln!("Server timed out!");
                ctx.stop();
                return;
            }
            act.writer.as_mut().unwrap().ping("");
            act.hb(ctx);
        });
    }
}

#[derive(Message)]
pub struct LogChunk(pub Vec<u8>);

/// Handle sending a chunk of compressed log data
impl Handler<LogChunk> for LogClient {
    type Result = ();

    fn handle(&mut self, msg: LogChunk, _ctx: &mut Context<Self>) {
        // Send a chunk using binary frame
        eprintln!("Send binary {:?}", msg.0);
        self.writer.as_mut().unwrap().binary(msg.0)
    }
}

/// Handle server websocket messages
impl StreamHandler<Message, ProtocolError> for LogClient {
    fn handle(&mut self, msg: Message, _ctx: &mut Context<Self>) {
        // This is mostly boilerplate. We don't expect any message back.
        match msg {
            Message::Pong(_) => self.heartbeat = SystemTime::now(),
            Message::Text(txt) => println!("Server: {:?}", txt),
            Message::Binary(bin) => println!("Binary: {:?}", bin),
            _ => (),
        }
    }

    fn started(&mut self, _ctx: &mut Context<Self>) {
        println!("Connected");
    }

    fn finished(&mut self, ctx: &mut Context<Self>) {
        println!("Server disconnected");
        ctx.stop()
    }
}
