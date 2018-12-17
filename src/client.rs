use actix::{Actor, ActorContext, Arbiter, AsyncContext, Context, Handler, StreamHandler, System};
use actix_web::ws::{Client, ClientWriter, Message, ProtocolError};
use futures::future::ok;
use futures::prelude::*;
use futures::sync::mpsc;
use std::time::Duration;

// This is the websocket client
pub struct LogClient(ClientWriter);

//
#[derive(Debug)]
pub enum Packet {
    Chunk(Vec<u8>),
}

impl LogClient {
    /// Spawns async connection to a remote websocket server,
    /// and returns a handle to a sender which will accept any
    /// kind of packet data.
    ///
    /// TODO: This wraps whole actix actor thing, and probably
    /// just exposing LogClient as an actor could be easier to use.
    fn connect(url: &str) -> mpsc::Sender<Packet> {
        // Bound of 1 to limit the rate of messages
        let (sender, receiver) = mpsc::channel(1);

        Arbiter::spawn(
            Client::new(url)
                .connect()
                .map_err(|e| {
                    eprintln!("Error: {}", e);
                    ()
                })
                .and_then(|(reader, writer)| {
                    let addr = LogClient::create(|ctx| {
                        LogClient::add_stream(reader, ctx);
                        LogClient(writer)
                    });

                    receiver
                        .for_each(move |message| {
                            // Here the packet is popped already
                            match message {
                                Packet::Chunk(data) => {
                                    println!("Received chunk of size {}", data.len());
                                    addr.send(LogChunk(data))
                                        .and_then(|_| {
                                            println!("Sent chunk message");
                                            ok(())
                                        })
                                        .map_err(|e| {
                                            eprintln!("Send error: {}", e);
                                            ()
                                        })
                                }
                            }
                        })
                        .and_then(|_| {
                            println!("Done");
                            ok(())
                        })
                        .into_future()
                }),
        );
        sender
    }
}

#[derive(Message)]
pub struct LogChunk(Vec<u8>);

impl Actor for LogClient {
    type Context = Context<Self>;

    fn started(&mut self, ctx: &mut Context<Self>) {
        // start heartbeats otherwise server will disconnect after 10 seconds
        self.hb(ctx)
    }

    fn stopped(&mut self, _: &mut Context<Self>) {
        println!("Disconnected");

        // Stop application on disconnect
        System::current().stop();
    }
}

impl LogClient {
    fn hb(&self, ctx: &mut Context<Self>) {
        // Do a heartbeat to keep the WS connection alive
        ctx.run_later(Duration::new(1, 0), |act, ctx| {
            act.0.ping("");
            act.hb(ctx);
            // TODO: client should also check for a timeout here, similar to the server code
        });
    }
}

/// Handle sending a chunk of compressed log data
impl Handler<LogChunk> for LogClient {
    type Result = ();

    fn handle(&mut self, msg: LogChunk, _ctx: &mut Context<Self>) {
        // Send a chunk using binary frame
        self.0.binary(msg.0)
    }
}

/// Handle server websocket messages
impl StreamHandler<Message, ProtocolError> for LogClient {
    fn handle(&mut self, msg: Message, _ctx: &mut Context<Self>) {
        // This is mostly boilerplate. We don't expect any message back.
        match msg {
            Message::Text(txt) => println!("Server: {:?}", txt),
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
