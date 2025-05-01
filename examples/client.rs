//! Мини‑клиент: посылает "hello" и печатает ответ
use anyhow::Result;
use bytes::BytesMut;
use mini_transport::{grpc::codec::encode_message, h2::{frame::Frame, sender::send_frame, parser::read_frames}};
use tokio::{net::TcpStream, time::{timeout, Duration}};

#[tokio::main]
async fn main() -> Result<()> {
    let mut stream = TcpStream::connect("127.0.0.1:50052").await?;
    println!("Connected to server at 127.0.0.1:50052");

    // Отправляем HTTP/2 preface
    stream.writable().await?;
    if let Err(e) = stream.try_write(b"PRI * HTTP/2.0\r\n\r\nSM\r\n\r\n") {
        eprintln!("Failed to send HTTP/2 preface: {}", e);
        return Ok(());
    }
    println!("Sent HTTP/2 preface");

    // Даем серверу время обработать preface
    tokio::time::sleep(Duration::from_millis(3000)).await;

    // Отправляем сообщение "hello"
    let mut payload = BytesMut::with_capacity(32);
    encode_message(b"hello", &mut payload);
    let data = Frame::Data { stream_id: 1, end_stream: true, payload: payload.freeze() };
    if let Err(e) = send_frame(&mut stream, data).await {
        eprintln!("Failed to send 'hello' to server: {}", e);
        return Ok(());
    }
    println!("Sent 'hello' to server");

    // Даем серверу время ответить
    tokio::time::sleep(Duration::from_millis(3000)).await;

    // Читаем ответ с тайм-аутом
    println!("Waiting for response...");
    match timeout(Duration::from_secs(20), read_frames(&mut stream)).await {
        Ok(Ok(Frame::Data { payload, .. })) => {
            println!("response: {}", String::from_utf8_lossy(&payload));
        }
        Ok(Ok(_)) => {
            eprintln!("Received unexpected frame type");
        }
        Ok(Err(e)) => {
            eprintln!("Failed to read response: {}", e);
        }
        Err(_) => {
            eprintln!("Timed out waiting for response");
        }
    }

    Ok(())
}