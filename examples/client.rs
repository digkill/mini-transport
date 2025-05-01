//! Мини‑клиент: посылает "hello" и печатает ответ
use anyhow::Result;
use bytes::BytesMut;
use mini_transport::{grpc::codec::encode_message, h2::{frame::Frame, sender::send_frame, parser::read_frames}};
use tokio::net::TcpStream;

#[tokio::main]
async fn main() -> Result<()> {
    let mut stream = TcpStream::connect("127.0.0.1:50051").await?;
    // Preface client connection string
    stream.writable().await?;
    stream.try_write(b"PRI * HTTP/2.0\r\n\r\nSM\r\n\r\n")?;

    let mut payload = BytesMut::with_capacity(32);
    encode_message(b"hello", &mut payload);
    let data = Frame::Data { stream_id: 1, end_stream: true, payload: payload.freeze() };
    send_frame(&mut stream, data).await?;

    if let Frame::Data { payload, .. } = read_frames(&mut stream).await? {
        println!("response: {}", String::from_utf8_lossy(&payload));
    }
    Ok(())
}