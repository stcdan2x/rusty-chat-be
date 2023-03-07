//! Methods that handle clients requests

use std::net::TcpStream;

use std::io::{
    Write,
    BufReader,
    BufRead,
};

use std::sync::{
    mpsc,
    Mutex,
    Arc,
};


pub fn handle_sent_messages(
    stream: TcpStream,
    sender: mpsc::Sender<String>,
) {
    /* create a buffer in order to read data sent through the stream */
    let mut buffer = BufReader::new(stream);

    let mut message = String::new();

    loop {

        /* blocking step to read data from the client stream */
        let request = buffer.read_line(&mut message);

        if request.is_err() {
            continue;
        }

        let message_copy = message.clone();
        let message_bytes = message_copy.as_bytes();

        const END_OF_LINE: u8 = 10;
        if message_bytes.get(0) == Some(&END_OF_LINE) {
            break;
        }

        let send_message = sender.send(message.to_string());
        if send_message.is_err() {
            break;
        }

        message.clear();
    }
}

pub fn receive_messages(
    receiver: mpsc::Receiver<String>,
    senders_mutex_pointer: Arc<Mutex<Vec<mpsc::Sender<String>>>>,
) {

    loop {

        /* blocking listening procedure for incoming messages */
        let message_result = receiver.recv();

        if message_result.is_err() {
            continue;
        }

        /* acquires the senders mutex, blocks until it is available */
        let guard = senders_mutex_pointer.lock().unwrap();

        let senders = &*guard;

        let message = message_result.unwrap();

        for sender in senders {
            sender.send(message.to_string())
                .expect("cannot send message");
        }
    }
}

pub fn send_to_client(
    mut stream: TcpStream,
    receiver: mpsc::Receiver<String>,
) {

    loop {

        /* the client receiver listens for messages,
           this is a blocking IO */
        let message_result = receiver.recv();

        if message_result.is_err() {
            continue;
        }
        
        let message = message_result.unwrap();
        let message_bytes = message.as_bytes();
        stream.write(message_bytes).unwrap();
    }
}
