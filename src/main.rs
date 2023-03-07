//! Basic TCP server

mod requests_handler;

use std::net::TcpListener;
use std::thread::spawn;
use std::sync::{
    Mutex,
    Arc,
};
use std::sync::mpsc::{
    Sender,
    Receiver,
    channel,
};

use requests_handler::{
    receive_messages,
    handle_sent_messages,
    send_to_client,
};

fn main() {

    /* create a TCP socket server */
    let listener = TcpListener::bind("0.0.0.0:9090").unwrap();

    /* creation of a channel for transmission */
    let (sender, receiver): (
        Sender<String>,
        Receiver<String>
    ) = channel();

    /* one sender per client */
    type Senders = Vec<Sender<String>>;
    let senders: Senders = Vec::new();

    
    let senders_mutex: Mutex<Senders> = Mutex::new(senders);

    let senders_mutex_pointer: Arc<Mutex<Senders>> = Arc::new(senders_mutex);

    let senders_mutex_pointer_copy = senders_mutex_pointer.clone();

    /* create a thread that listens for all incoming messages
       and forward them to every connected clients */
    spawn(|| {
        receive_messages(
            receiver,
            senders_mutex_pointer,
        );
    });

    for income in listener.incoming() {

        if income.is_err() {
            continue;
        }

        let stream = income.unwrap();

        /* get the address and port of the remove peer of the given client */
        let client_address = stream.peer_addr()
            .unwrap();

        println!(
            "New client connected: {}",
            client_address,
        );

        let stream_copy = stream.try_clone()
            .expect("Cannot clone TCP stream");

        /* create a new sender from the channel sender */
        let sender_copy = sender.clone();

        /* create a thread that handles sent messages from the new client */
        spawn(|| {
            handle_sent_messages(
                stream_copy,
                sender_copy,
            );
        });

        let (
            client_sender,
            client_receiver
        ): (
            Sender<String>,
            Receiver<String>
        ) = channel();

        /* create one thread per client */
        spawn(|| {
            send_to_client(
                stream,
                client_receiver,
            );
        });

        let mut guard = senders_mutex_pointer_copy.lock().unwrap();

        let mut senders = &mut *guard;

        senders.push(client_sender);
    }
}
