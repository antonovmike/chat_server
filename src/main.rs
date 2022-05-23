// SERVER
#![allow(unused)]
use std::io::{ErrorKind, Read, Write};
use std::net::TcpListener;
use std::sync::mpsc;
use std::thread;
use std::time::Duration;
use colored::Colorize;

const LOCAL: &str = "127.0.0.1:6000"; // Connection address
const MESSAGE_SIZE: usize = 32; // Limit of chars. Extra chars would not be printed
const USER_NAME_SIZE: usize = 16;
// Example: If MESSAGE_SIZE = 8 and user send "123456789", server will print "12345678"

fn main() {
    let server = TcpListener::bind(LOCAL).expect("Listener failed to bind");
    server.set_nonblocking(true).expect("Failed to initialize non-blocking");

    let mut clients = vec![];
// Channels have two endpoints: the `Sender<T>` and the `Receiver<T>`,
// where `T` is the type of the message to be transferred
// (type annotation is superfluous)
// https://doc.rust-lang.org/rust-by-example/std_misc/channels.html
    let (tx, rx) = mpsc::channel::<String>();
    loop {
        if let Ok((mut socket, addr)) = server.accept() {
			//let name = "USER".to_string();
            println!("{}", format!("User {} connected", addr).yellow());
            println!("{}", format!("User's address is {}", addr).yellow());
            let tx = tx.clone();
            clients.push(socket.try_clone().expect("Failed to clone client"));
// Each thread will send its id via the channel
            thread::spawn(move || loop {
                let mut buff_name = vec![0; USER_NAME_SIZE];
                match socket.read_exact(&mut buff_name) {
                    Ok(_) => {
                        let user_name = buff_name.into_iter().take_while(|&x| x != 0).collect::<Vec<_>>();
                        let user_name = String::from_utf8(user_name).expect("Invalid utf8 message");
                        print!("{}", format!("{} said: ", user_name).italic().on_green());
                        tx.send(user_name).expect("Failed to send message to rx"); // MOVED HERE
                    },
                    Err(ref err) if err.kind() == ErrorKind::WouldBlock => (),
                    Err(_) => {
                        println!("{}", format!("Closing connection with {}", addr).on_red());
                        break;
                    }
                }
                sleep();
                
                let mut buff_message = vec![0; MESSAGE_SIZE];
                match socket.read_exact(&mut buff_message) {
                    Ok(_) => {
                        let users_message = buff_message.into_iter().take_while(|&x| x != 0).collect::<Vec<_>>();
                        let users_message = String::from_utf8(users_message).expect("Invalid utf8 message");
                        println!("{}", format!("{}", users_message).italic().on_green());
                        tx.send(users_message).expect("Failed to send message to rx"); // MOVED HERE
                    },
                    Err(ref err) if err.kind() == ErrorKind::WouldBlock => (),
                    Err(_) => {
                        println!("{}", format!("Closing connection with {}", addr).on_red());
                        break;
                    }
                }

                sleep();
            });
        }
 //try_recv attempts to receive a message from the channel without blocking
 //https://docs.rs/crossbeam-channel/0.1.3/crossbeam_channel/struct.Receiver.html
        if let Ok(user_name) = rx.try_recv() {
            clients = clients.into_iter().filter_map(|mut client| {
                let mut buff_name = user_name.clone().into_bytes();
                buff_name.resize(MESSAGE_SIZE, 0);
                client.write_all(&buff_name).map(|_| client).ok()
            })
                .collect::<Vec<_>>();
        }
        if let Ok(users_message) = rx.try_recv() {
            clients = clients.into_iter().filter_map(|mut client| {
                let mut buff_message = users_message.clone().into_bytes();
                buff_message.resize(MESSAGE_SIZE, 0);
                client.write_all(&buff_message).map(|_| client).ok()
            })
                .collect::<Vec<_>>();
        }

        sleep();
    }
}

fn sleep() {
    thread::sleep(Duration::from_millis(100));
}
