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

#[cfg(test)]
use actix::actors::mocker::Mocker;

#[cfg(test)]
pub type LogClient = Mocker<LogClientAct>;
#[cfg(not(test))]
pub type LogClient = LogClientAct;

// This is the websocket client
pub struct LogClientAct {
    url: String,
    backoff: ExponentialBackoff,
    writer: Option<ClientWriter>,
    // Time when a ping was sent
    heartbeat: SystemTime,
}

impl Default for LogClientAct {
    fn default() -> LogClientAct {
        LogClientAct {
            url: String::new(),
            backoff: ExponentialBackoff::default(),
            writer: None,
            heartbeat: SystemTime::now(),
        }
    }
}

impl actix::Supervised for LogClientAct {
    fn restarting(&mut self, ctx: &mut Context<LogClientAct>) {
        println!("restarting");
    }
}

impl Actor for LogClientAct {
    type Context = Context<Self>;

    fn started(&mut self, ctx: &mut Context<Self>) {
        println!("Started");
        if !self.url.is_empty() {
            ctx.notify(Connect(self.url.clone()))
        }
    }
}

impl LogClientAct {
    fn hb(&self, ctx: &mut Context<Self>) {
        ctx.run_later(Duration::new(1, 0), |mut act, ctx| {
            eprintln!("Heartbeat elapsed: {:?}", act.heartbeat.elapsed().unwrap());
            if act.heartbeat.elapsed().unwrap() >= Duration::from_secs(5) {
                eprintln!("Server timed out!");
                ctx.stop();
                return;
            }
            if let Some(ref mut writer) = act.writer.as_mut() {
                writer.ping("");
            }
            act.hb(ctx);
        });
    }
}

#[derive(Message)]
pub struct Connect(pub String);
impl Handler<Connect> for LogClientAct {
    type Result = ();

    fn handle(&mut self, msg: Connect, ctx: &mut Context<Self>) {
        // Send a chunk using binary frame
        self.url = msg.0.clone();
        Client::new(msg.0.clone())
            .connect()
            .into_actor(self)
            .map(move |(reader, writer), mut act, mut ctx| {
                // Reset the backoff timer
                act.backoff.reset();
                // Add reader stream
                LogClientAct::add_stream(reader, ctx);
                // Keep the writer for later
                act.writer = Some(writer);
                // Start pinging
                act.heartbeat = SystemTime::now();
                act.hb(&mut ctx);
            })
            .map_err(|err, act, ctx| {
                eprintln!("Unable to connect to: {}", err);
                if let Some(timeout) = act.backoff.next_backoff() {
                    eprintln!("Next timeout: {:?}", timeout);
                    ctx.run_later(timeout, |_, ctx| ctx.stop());
                }
            })
            .wait(ctx);
    }
}

#[derive(Message)]
pub struct LogChunk(pub Vec<u8>);

/// Handle sending a chunk of compressed log data
impl Handler<LogChunk> for LogClientAct {
    type Result = ();

    fn handle(&mut self, msg: LogChunk, _ctx: &mut Context<Self>) {
        // Send a chunk using binary frame
        eprintln!("Send binary {:?}", msg.0);
        if let Some(ref mut writer) = self.writer.as_mut() {
            writer.binary(msg.0)
        }
    }
}

/// Handle server websocket messages
impl StreamHandler<Message, ProtocolError> for LogClientAct {
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
