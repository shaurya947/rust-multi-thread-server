use std::time::Duration;

use async_std::{
    fs,
    io::BufReader,
    net::{TcpListener, TcpStream},
    prelude::*,
    sync::Arc,
    task,
};
use futures::{future::try_join, stream::StreamExt};

pub async fn run() -> std::io::Result<()> {
    // read sample html response as string
    let (body_ok, body_404) = try_join(
        fs::read_to_string("hello.html"),
        fs::read_to_string("404.html"),
    )
    .await?;
    let (resp_body_ok, resp_body_404) = (Arc::new(body_ok), Arc::new(body_404));

    let listener = TcpListener::bind("127.0.0.1:8080").await?;
    let mut incoming = listener.incoming();

    // accept connections and process them serially
    while let Some(stream) = incoming.next().await {
        let (stream, resp_body_ok, resp_body_404) = (
            stream?,
            Arc::clone(&resp_body_ok),
            Arc::clone(&resp_body_404),
        );
        task::spawn(async move {
            handle_client(stream, &resp_body_ok, &resp_body_404).await;
        });
    }
    Ok(())
}

async fn handle_client(mut stream: TcpStream, resp_body_ok: &str, resp_body_404: &str) {
    let buf_reader = BufReader::new(&stream);
    let request_line = buf_reader.lines().next().await.unwrap().unwrap();

    let (status_line, content) = match &request_line[..] {
        "GET / HTTP/1.1" => ("HTTP/1.1 200 OK", resp_body_ok),
        "GET /sleep HTTP/1.1" => {
            task::sleep(Duration::from_secs(5)).await;
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
    stream.write_all(response.as_bytes()).await.unwrap();
}
