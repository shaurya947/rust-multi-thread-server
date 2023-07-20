use std::{
    format, fs,
    io::{BufRead, BufReader, Write},
    net::{TcpListener, TcpStream},
    sync::Arc,
    thread,
    time::Duration,
};

pub fn run() -> std::io::Result<()> {
    // read sample html response as string
    let (resp_body_ok, resp_body_404) = (
        Arc::new(fs::read_to_string("hello.html")?),
        Arc::new(fs::read_to_string("404.html")?),
    );

    let listener = TcpListener::bind("127.0.0.1:8080")?;

    // accept connections and process them in threads
    for stream in listener.incoming() {
        let (stream, resp_body_ok, resp_body_404) = (
            stream?,
            Arc::clone(&resp_body_ok),
            Arc::clone(&resp_body_404),
        );
        thread::spawn(move || handle_client(stream, &resp_body_ok, &resp_body_404));
    }
    Ok(())
}

fn handle_client(mut stream: TcpStream, resp_body_ok: &str, resp_body_404: &str) {
    let buf_reader = BufReader::new(&stream);
    let request_line = buf_reader.lines().next().unwrap().unwrap();

    let (status_line, content) = match &request_line[..] {
        "GET / HTTP/1.1" => ("HTTP/1.1 200 OK", resp_body_ok),
        "GET /sleep HTTP/1.1" => {
            thread::sleep(Duration::from_secs(5));
            ("HTTP/1.1 200 OK", resp_body_ok)
        }
        _ => ("HTTP/1.1 404 NOT FOUND", resp_body_404),
    };

    let response = format!(
        "{}\r\nContent-Length: {}\r\n\r\n{}",
        status_line,
        content.len(),
        content
    );
    stream.write_all(response.as_bytes()).unwrap();
}
