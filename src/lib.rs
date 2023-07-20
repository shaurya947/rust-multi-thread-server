use std::{
    collections::VecDeque,
    format, fs,
    io::{BufRead, BufReader, Write},
    net::{TcpListener, TcpStream},
    sync::{Arc, Mutex},
    thread,
    time::Duration,
};

const MAX_CONCURRENT_THREADS: i32 = 4;

pub fn run() -> std::io::Result<()> {
    let tcp_queue = Arc::new(Mutex::new(VecDeque::new()));

    // read sample html response as string
    let (resp_body_ok, resp_body_404) = (
        Arc::new(fs::read_to_string("hello.html")?),
        Arc::new(fs::read_to_string("404.html")?),
    );

    let listener = TcpListener::bind("127.0.0.1:8080")?;

    // launch forever-alive threads ready to process requests
    (0..MAX_CONCURRENT_THREADS).for_each(|_| {
        let (tcp_queue, resp_body_ok, resp_body_404) = (
            Arc::clone(&tcp_queue),
            Arc::clone(&resp_body_ok),
            Arc::clone(&resp_body_404),
        );
        thread::spawn(move || {
            loop {
                // attempt to dequeue a TCP stream
                let mut tcp_stream = None;
                {
                    let mut lock = tcp_queue.try_lock();
                    if let Ok(ref mut tcp_queue) = lock {
                        tcp_stream = tcp_queue.pop_front();
                    }
                }

                if let Some(tcp_stream) = tcp_stream {
                    handle_client(tcp_stream, &resp_body_ok, &resp_body_404);
                } else {
                    thread::sleep(Duration::from_millis(100));
                }
            }
        });
    });

    // accept connections and process them in threads
    for stream in listener.incoming() {
        tcp_queue.lock().unwrap().push_back(stream?);
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
