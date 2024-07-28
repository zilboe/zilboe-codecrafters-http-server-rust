// Uncomment this block to pass the first stage
use std::{env, fs::{self, File}, io::{Read, Write}, net::{TcpListener, TcpStream}, path, thread};
use flate2::write::{GzEncoder};
use flate2::Compression;
use nom::AsChar;
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
    let mut is_gzip: bool = false;
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
    
    for recv_split_one in &recv_split {
        if recv_split_one.starts_with("Accept-Encoding:") {
            let recv_split_skip_encoding_head = "Accept-Encoding:".len();
            let recv_split_skip_encoding = &recv_split_one[recv_split_skip_encoding_head..];
            let recv_split_one_encoding: Vec<&str> = recv_split_skip_encoding.split(',').collect();
            for recv_encoding_split in recv_split_one_encoding {
                if recv_encoding_split.trim().starts_with("gzip") {
                    is_gzip = true;
                }
            }
        }
    };
    if recv_split[0].starts_with("GET /index.html") {
        write_buff.push_str("HTTP/1.1 200 OK\r\n\r\n");
    } else if recv_split[0].starts_with("GET /echo/"){

        write_buff.push_str("HTTP/1.1 200 OK\r\n");
        write_buff.push_str("Content-Type: text/plain\r\n");

        if is_gzip {
            write_buff.push_str("Content-Encoding: gzip\r\n");
        }

        let echo_head_len = "GET /echo/".len();
        let echo_head_str = &recv_split[0][echo_head_len..];
        let echo_str_split_space: Vec<&str> = echo_head_str.split(' ').collect();
        let echo_str_len = echo_str_split_space[0].len();
        
        

        if is_gzip {
            let mut e = GzEncoder::new(Vec::new(), Compression::default());
            e.write_all(echo_str_split_space[0].as_bytes()).expect("Turn Buff Gzip U8 Error");
            let write_encoder: Vec<u8> = e.finish().expect("gzip Error");
            let write_echo_str = format!("Content-Length: {}\r\n\r\n",write_encoder.len());
            write_buff.push_str(&write_echo_str);
            for i in write_encoder {
                write_buff.push(i.as_char());
            }
        } else {
            let write_echo_str = format!("Content-Length: {}\r\n\r\n",echo_str_len);
            write_buff.push_str(&write_echo_str);
            write_buff.push_str(&echo_str_split_space[0]);
        }
        
    } else if recv_split[0].starts_with("GET /user-agent"){
        for user_agent_split in recv_split {
            if user_agent_split.starts_with("User-Agent: ") {
                write_buff.push_str("HTTP/1.1 200 OK\r\n");
                write_buff.push_str("Content-Type: text/plain\r\n");

                if is_gzip {
                    write_buff.push_str("Content-Encoding: gzip\r\n");
                }

                let user_agent_head_len: usize = "User-Agent: ".len();
                let user_agent_head_split: Vec<&str> = user_agent_split[user_agent_head_len..].split(' ').collect();
                let user_agent_head_len = user_agent_head_split[0].len();

                if is_gzip {
                    let mut e = GzEncoder::new(Vec::new(), Compression::default());
                    e.write_all(user_agent_head_split[0].as_bytes()).expect("Turn Buff Gzip U8 Error");
                    let write_encoder: Vec<u8> = e.finish().expect("gzip Error");
                    let write_user_agent_str = format!("Content-Length: {}\r\n\r\n", write_encoder.len());
                    write_buff.push_str(&write_user_agent_str);
                    for i in write_encoder {
                        write_buff.push(i.as_char());
                    }
                } else {
                    let write_user_agent_str = format!("Content-Length: {}\r\n\r\n", user_agent_head_len);
                    write_buff.push_str(&write_user_agent_str);
                    write_buff.push_str(&user_agent_head_split[0]);
                }


                
                break;
            }
        };

    } else if recv_split[0].starts_with("GET /files") {
        let arg: Vec<String> = env::args().collect();   //env arg
        let mut file_name_head = String::from(&arg[2]);
        let file_head_len = "GET /files".len();
        let file_head_str = &recv_split[0][file_head_len..];
        let file_head_split: Vec<&str> = file_head_str.split(' ').collect();
        file_name_head.push_str(file_head_split[0]);
        let path_file = path::Path::new(&file_name_head);
        if path_file.exists() {
            write_buff.push_str("HTTP/1.1 200 OK\r\n");
            write_buff.push_str("Content-Type: application/octet-stream\r\n");

            if is_gzip {
                write_buff.push_str("Content-Encoding: gzip\r\n");
            }

            let path_file_open = fs::OpenOptions::new().read(true).open(file_name_head);
            let mut path_file_open = match path_file_open {
                Ok(file) => file,
                Err(_) => {
                    println!("open file error");
                    return;
                }
            };
            let mut file_open_buff: Vec<u8> = Vec::new();
            match path_file_open.read_to_end(&mut file_open_buff) {
                Ok(_) => {
                    let file_buffer_len = file_open_buff.len();
                    let file_open_buff_to_string =  String::from_utf8_lossy(&file_open_buff);

                    if is_gzip {
                        let mut e = GzEncoder::new(Vec::new(), Compression::default());
                        e.write_all(&file_open_buff).expect("Turn Buff Gzip U8 Error");
                        let write_encoder: Vec<u8> = e.finish().expect("gzip Error");
                        let write_file_buff = format!("Content-Length: {}\r\n\r\n", write_encoder.len());
                        write_buff.push_str(&write_file_buff);
                        for i in write_encoder {
                            write_buff.push(i.as_char());
                        }
                    } else {
                        let write_file_buff = format!("Content-Length: {}\r\n\r\n", file_buffer_len);
                        write_buff.push_str(&write_file_buff);
                        write_buff.push_str(&file_open_buff_to_string);
                    }

                }
                Err(_) => {
                    println!("read file to buffer error");
                    return;
                }
            }

        } else {
            write_buff.push_str("HTTP/1.1 404 Not Found\r\n\r\n");
        }
    } else if recv_split[0].starts_with("POST /files/") {
        let arg: Vec<String> = env::args().collect();   //env arg
        let file_name_head = String::from(&arg[2]);     //Get ENV Path
        let post_head_len = "POST /files/".len();           
        let post_head_str_split = &recv_split[0][post_head_len..];      //Get POST request index
        let post_head_str: Vec<&str> = post_head_str_split.split(' ').collect();    //Get POST request filename

        let post_head_filename = file_name_head + &post_head_str[0];    //Get The Whole Filename including path
        
        
        let request_post_content = recv_split.len();    //Get POST request Content
        let request_post_content = recv_split[request_post_content-1];  //Get POST content
        //println!("{}", post_head_filename);
        for get_post_recv_head in recv_split {
            if get_post_recv_head.starts_with("Content-Length:") {
                let skip_post_content_len = "Content-Length: ".len();
                let get_post_content_size = &get_post_recv_head[skip_post_content_len..];
                let get_post_content_size: usize = get_post_content_size.parse().expect("POST: Get content-len Error");

                let mut create_file: File = fs::File::create(post_head_filename.clone()).expect("create file error");
                let request_post_content = &request_post_content[..get_post_content_size];
                create_file.write(request_post_content.as_bytes()).expect("POST: write data to file error");
                
                write_buff.push_str("HTTP/1.1 201 Created\r\n\r\n");
            }
        }
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
                thread::spawn(|| {handle_connect(_stream)});
            }
            Err(e) => {
                println!("error: {}", e);
            }
        }
    };
}
