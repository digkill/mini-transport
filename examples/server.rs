use anyhow::Result;
use bytes::BytesMut;
use mini_transport::{grpc::codec::encode_message, h2::{frame::Frame, sender::send_frame, parser::read_frames}};
use tokio::net::TcpListener;

#[tokio::main]
async fn main() -> Result<()> {
    let listener = TcpListener::bind("127.0.0.1:50051").await?;
    println!("Server listening on 127.0.0.1:50051");

    loop {
        let (mut stream, _) = listener.accept().await?;
        tokio::spawn(async move {
            // Handle client preface (HTTP/2 connection string)
            let mut buf = BytesMut::with_capacity(24);
            if let Frame::Data { payload, .. } = read_frames(&mut stream).await.unwrap() {
                println!("Received: {}", String::from_utf8_lossy(&payload));
                // Respond with a message
                let mut response = BytesMut::with_capacity(32);
                encode_message(b"world", &mut response);
                let frame = Frame::Data {
                    stream_id: 1,
                    end_stream: true,
                    payload: response.freeze(),
                };
                send_frame(&mut stream, frame).await.unwrap();
            }
        });
    }
}