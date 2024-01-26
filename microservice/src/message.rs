use std::{net::SocketAddr, ops::ControlFlow};

use axum::extract::ws::Message;

pub fn process_message(message: Message, address: SocketAddr) -> ControlFlow<(), ()> {
    match message {
        Message::Text(t) => {
            println!(">>> {address} sent str: {t:?}");
        }

        Message::Binary(d) => {
            println!(">>> {} sent {} bytes: {:?}", address, d.len(), d);
        }

        Message::Close(c) => {
            if let Some(cf) = c {
                println!(
                    ">>> {} sent close with code {} and reason `{}`",
                    address, cf.code, cf.reason
                );
            } else {
                println!(">>> {address} somehow sent close message without CloseFrame");
            }
            return ControlFlow::Break(());
        }

        Message::Pong(v) => {
            if v == vec![1, 2, 3] {
                println!(">>> {address} sent ping with {v:?} and got expected pong");
            } else {
                println!(
                    ">>> {address} sent ping with {v:?} and got unexpected pong",
                    address = address,
                    v = v
                );
            }
        }
        // You should never need to manually handle Message::Ping, as axum's websocket library
        // will do so for you automagically by replying with Pong and copying the v according to
        // spec. But if you need the contents of the pings you can see them here.
        Message::Ping(v) => {
            println!(">>> {address} sent ping with {v:?}");
        }
    }
    ControlFlow::Continue(())
}
