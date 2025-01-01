// Uncomment this block to pass the first stage
use flate2::write::{self, GzEncoder};
use flate2::Compression;
use nom::{FindSubstring};
use std::{path::Path, ptr::eq, env, io};
use httparse::Request;
use tokio::{
    io::{AsyncReadExt, AsyncWriteExt},
    net::{TcpListener, TcpStream},
};

// we need some error process such as 404 page
// so we add this MyError including the err_data
// that we can delivery the code
struct MyError {
    message: String,
    err_data: Vec<u8>,
}
impl From<io::Error> for MyError {
    fn from(error: io::Error) -> Self{
        MyError { 
            message: error.to_string(), 
            err_data: vec![],
        }
    }
}

/// HTTP Config, it doesn't require much configuration
struct WebRequest{
    http_stream: TcpStream,
    keep_alive: bool,
    is_gzip: bool,
    uri_path: Option<String>,
}

impl WebRequest {
    fn new(stream: TcpStream) -> Self {
        WebRequest {
            http_stream: stream,
            keep_alive: false,
            is_gzip: false,
            uri_path: None,
        }
    }

    // The entire access path and access parameter 
    // processing need to determine the local file 
    // and needs to fill in the content-type tag
    fn get_path(&mut self, uri_path: &str) -> Result<Vec<u8>, MyError> {
        let mut uri_path_name = String::from(uri_path);

        if uri_path.ends_with('/') {    //??? i think this need considering the Multiple '/'
            uri_path_name += "index.html";
        }
        
        match self.set_file_path_isexist(&uri_path_name) {
            Ok(mut write_buff) => {
                //then i need to analy the request file type
                match uri_path_name.find('.') {
                    Some(size) => {
                        write_buff.extend_from_slice(b"Content-Type: ");
                        let file_type = match &uri_path_name[size..] {
                            ".html" => "text/html",
                            ".css" => "text/css",
                            ".bmp" => "application/x-bmp",
                            ".img" => "application/x-img",
                            ".jpe" => "image/jpeg",
                            ".jpeg" => "image/jpeg",
                            ".jpg" => "image/jpeg",
                            ".js" => "application/x-javascript",
                            ".mp4" => "video/mpeg4",
                            ".xml" => "	text/xml",
                            ".xquery" => "text/xml",
                            ".xsl" => "text/xml",
                            _ => "application/octet-stream",
                        };
                        write_buff.extend_from_slice(file_type.as_bytes());
                    },
                    None => {
                        write_buff.extend_from_slice(b"application/octet-stream");
                    }
                };
                write_buff.extend_from_slice(b"\r\n");
                Ok(write_buff)
            },

            Err(e) => {
                Err(e)
            }
        }//404 NO FOUND
    }

    // need to check that the file exists and pass 
    // the full file name to the uri_path parameter
    // and pass the response code 200 and 404 
    fn set_file_path_isexist(&mut self, file_name: &str) -> Result<Vec<u8>, MyError> {
        let env_arg: Vec<String> = env::args().collect();
        let mut response_code: Vec<u8> = Vec::new();
        if env_arg.len() < 2 {
            response_code.extend_from_slice(b"HTTP/1.1 404 OK\r\n\r\n");
            Err(MyError{
                message: "404 NO FOUND".to_string(),
                err_data: response_code,
            })
        } else {
            let take_uri_path = file_name;
            let env_file_path = env_arg[1].as_str().to_owned() + take_uri_path;
            let path_file: &Path = Path::new(&env_file_path);
            if path_file.exists() {
                self.uri_path = self.uri_path.replace(env_file_path);
                response_code.extend_from_slice(b"HTTP/1.1 200 OK\r\n");
                Ok(response_code)
            } else {
                response_code.extend_from_slice(b"HTTP/1.1 404 OK\r\n\r\n");
                Err(MyError{
                    message: "404 FILE NO FOUND".to_string(),
                    err_data: response_code,
                })
            }
        }
    }

    // Determine whether to enable gzip compression
    fn set_gzip_config(&mut self, recv_buff: &str) -> io::Result<()> {
        self.is_gzip = recv_buff.find_substring("gzip").is_some();
        Ok(())
    }

    // Check whether the keepalive parameter exists
    fn set_keepalive_config(&mut self, recv_buff: &str) -> io::Result<()> {
        self.keep_alive = eq(recv_buff, "true");
        Ok(())
    }

    // The entire response header portion is processed from the 
    // time the recv_buffer is received until the data is finally processed 
    // and the data to be sent is returned.
    // Use the parsing method of httparse library, it is zero copy parsing,
    // so the speed will be relatively faster, 
    // at present is about the use of each request processing if, 
    // then consider modifying to a more standard processing
    fn fill_response(&mut self, recv_buff: &[u8]) -> Vec<u8> {
        // we need parse the header, so i alloc httparse named header
        let mut header = [httparse::EMPTY_HEADER; 32];
        let mut request = Request::new(&mut header);

        // execute parse
        request.parse(recv_buff).unwrap();

        // the httparse go end And i set the path into the WebRequest's uri_path,
        // actually not noly the uri path, the header includes metho version and key-value
        // but the path we need to handle special case
        let mut write_buff = match self.get_path(request.path.unwrap()) {
            Ok(write_buff) => write_buff,
            Err(e) => {
                println!("{}", e.message);
                e.err_data
            }
        };

        for header_item in header {
            if header_item.name.eq_ignore_ascii_case("Accept-Encoding") {
                    let _ = self.set_gzip_config(std::str::from_utf8(header_item.value).unwrap());
            }
            if header_item.name.eq_ignore_ascii_case("keep-alive") {
                    let _ = self.set_keepalive_config(std::str::from_utf8(header_item.value).unwrap());
            }
        }
        
        // Whether to select file compression
        if self.is_gzip {
            write_buff.extend_from_slice(b"Content-Encoding: gzip\r\n");
        }
        
        write_buff
    }

    async fn send(&mut self, write_buff: &[u8]) -> io::Result<()> {
        let _ = self.http_stream.write_all(write_buff).await;
        Ok(())
    }

    async fn close(&mut self) {
        // if !self.keep_alive {
            let _ = self.http_stream.shutdown().await;
        // }
    }

}

async fn process_content(stream: TcpStream, recv_buff: &[u8]){
    let mut one_request = WebRequest::new(stream);
    let send_buffer= one_request.fill_response(recv_buff);
    let _ = one_request.send(&send_buffer).await;
    one_request.close().await;
}

async fn handle_connect(mut stream: TcpStream) {
    let mut read_buff: [u8; 512] = [0; 512];
    let _ = stream.read(&mut read_buff).await;
    process_content(stream, &read_buff).await;
    
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
