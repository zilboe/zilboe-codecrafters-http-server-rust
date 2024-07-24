// Uncomment this block to pass the first stage
use std::{io::{Read, Write}, net::{TcpListener, TcpStream}};

fn handle_connect(mut stream: TcpStream) {
    let mut read_buff: [u8; 1024] = [0; 1024];
    let read_res = stream.read(&mut read_buff);
    match read_res {
        Ok(_) => {

        }
        Err(_) => {
            eprintln!("recv from connect error");
        }
    }
    let response = String::from("HTTP/1.1 200 OK\r\n\r\n");
    let send_res = stream.write_all(response.as_bytes());
    match send_res {
        Ok(_) => {

        }
        Err(_) => {
            eprintln!("response error");
        }
    };
}

fn main() {
    // You can use print statements as follows for debugging, they'll be visible when running tests.
    println!("Logs from your program will appear here!");

    // Uncomment this block to pass the first stage
    //
    let listener = TcpListener::bind("127.0.0.1:4221").unwrap();
    //
    for stream in listener.incoming() {
        match stream {
            Ok(_stream) => {
                println!("accepted new connection");
                handle_connect(_stream);
            }
            Err(e) => {
                println!("error: {}", e);
            }
        }
    }
}
