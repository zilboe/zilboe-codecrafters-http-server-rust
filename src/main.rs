// Uncomment this block to pass the first stage
use std::{io::{Read, Write}, net::{TcpListener, TcpStream}};

use itertools::Itertools;
fn stream_send(mut stream: TcpStream, buff: String) {
    match stream.write_all(buff.as_bytes()) {
        Ok(_) => {

        }
        Err(_) => {
            eprintln!("response error");
        }
    }
}


fn handle_connect(mut stream: TcpStream) {
    let mut read_buff: [u8; 1024] = [0; 1024];
    let read_res = stream.read(&mut read_buff);
    let recv_string = String::from_utf8((&read_buff).to_vec()).expect("utf-8 to string error");
    //println!("{}", recv_string);
    match read_res {
        Ok(_) => {
        }
        Err(_) => {
            eprintln!("recv from connect error");
        }
    }
    let recv_split: Vec<&str> = recv_string.split("\r\n").collect();
    let mut write_buff = String::new();
   
    if recv_split[0].starts_with("GET /index.html") {
        write_buff.push_str("HTTP/1.1 200 OK\r\n\r\n");
    } else if recv_split[0].starts_with("GET /echo/"){

        write_buff.push_str("HTTP/1.1 200 OK\r\n");
        write_buff.push_str("Content-Type: text/plain\r\n");

        let echo_head_len = "GET /echo/".len();
        let echo_head_str = &recv_split[0][echo_head_len..];
        let echo_str_split_space: Vec<&str> = echo_head_str.split(' ').collect_vec();
        let echo_str_len = echo_str_split_space[0].len();
        //println!("{}", echo_str_split_space[0]);
        let write_echo_str = format!("Content-Length: {}\r\n\r\n{}",echo_str_len, echo_str_split_space[0]);
        write_buff.push_str(&write_echo_str);
        //println!("{}", write_buff);
    } else if recv_split[0].starts_with("GET /user-agent"){
        for user_agent_split in recv_split {
            if user_agent_split.starts_with("User-Agent: ") {
                write_buff.push_str("HTTP/1.1 200 OK\r\n");
                write_buff.push_str("Content-Type: text/plain\r\n");

                let user_agent_head_len: usize = "User-Agent: ".len();
                let user_agent_head_split: Vec<&str> = user_agent_split[user_agent_head_len..].split(' ').collect_vec();
                let user_agent_head_len = user_agent_head_split[0].len();
                let write_user_agent_str = format!("Content-Length: {}\r\n\r\n{}", user_agent_head_len, user_agent_head_split[0]);
                write_buff.push_str(&write_user_agent_str);
                break;
            }
        };

    } else {
        if recv_split[0].starts_with("GET / ") {
            write_buff.push_str("HTTP/1.1 200 OK\r\n\r\n");
        } else {
            write_buff.push_str("HTTP/1.1 404 Not Found\r\n\r\n");
        }
    }

    stream_send(stream, write_buff);
    
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
