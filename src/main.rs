// Uncomment this block to pass the first stage
use flate2::write::{self, GzEncoder};
use flate2::Compression;
use nom::FindSubstring;
use std::arch::x86_64::_CMP_FALSE_OQ;
use std::io::{self, Read, Write};
use anyhow::Result;
use httparse::{Request, EMPTY_HEADER};
use tokio::{
    io::{AsyncReadExt, AsyncWriteExt},
    net::{TcpListener, TcpStream},
};

/// HTTP Config
struct WebRequest{
    stream: TcpStream,
    // request_parse: Request<'a, 'a>,
    keep_alive: bool,
    is_gzip: bool,
    uri_path: Option<String>,
}

impl WebRequest {
    fn new(stream: TcpStream) -> Self {
        // let mut header = [httparse::EMPTY_HEADER; 8];
        WebRequest {
            stream: stream,
            // request_parse: Request::new(&mut header),
            keep_alive: false,
            is_gzip: false,
            uri_path: None,
        }
    }
    fn send(&mut self, write_buff: &[u8]) -> io::Result<()> {
        let _ = self.stream.write_all(write_buff);
        Ok(())
    }

    ///Parse URI PATH Str,Aviod ending '/'
    fn get_path(&mut self, uri_path: &str) -> io::Result<()> {
        let mut uri_path_name = String::from(uri_path);
        if uri_path.len() == 1 {
            self.uri_path = Some(uri_path_name);
        } else {
            if uri_path.ends_with('/') {    //需要考虑多个/结尾的情况
                uri_path_name = uri_path_name + "index.html";
            }
            self.uri_path = Some(uri_path_name);
        }
        // println!("uri name {:?}", self.uri_path);
        Ok(())
    }

    fn set_Gzip(&mut self, encoding: &str) -> io::Result<()> {
        if let Some(_) = encoding.find_substring("gzip") {
            self.is_gzip = true
        } else {
            self.is_gzip = false
        }
        Ok(())
    }

    fn close(&mut self) -> io::Result<()> {
        if !self.keep_alive {
            let _ = self.stream.shutdown();
        }
        Ok(())
    }
}
fn process_content(stream: TcpStream, recv_buff: &[u8]){
    let mut one_request = WebRequest::new(stream);
    
    let mut header = [httparse::EMPTY_HEADER; 64];
    let mut request = Request::new(&mut header);
    request.parse(&recv_buff).unwrap();

    let _ = one_request.get_path(&request.path.unwrap());



    // println!("uri_path {:?}", one_request.uri_path.unwrap());


    let _ = one_request.close();
}

async fn handle_connect(mut stream: TcpStream) {
    let mut read_buff: [u8; 512] = [0; 512];
    let _ = stream.read(&mut read_buff).await;
    process_content(stream, &mut read_buff);
    
    // let _ = stream.shutdown().await;
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // You can use print statements as follows for debugging, they'll be visible when running tests.
    println!("Logs from your program will appear here!");

    // Uncomment this block to pass the first stage
    //
    let listener: TcpListener = TcpListener::bind("127.0.0.1:4221").await?;
    //
    loop {
        let (socket, _) = listener.accept().await?;
        tokio::spawn(async move {
            handle_connect(socket).await;
        });
    }
}
