//! Запускает упрощенный HTTP/2 echo-сервер на 127.0.0.1:50052
use anyhow::Result;
use bytes::BytesMut;
use mini_transport::{grpc::codec::encode_message, h2::{frame::Frame, sender::send_frame, parser::read_frames}};
use tokio::{net::TcpListener, time::Duration};

#[tokio::main]
async fn main() -> Result<()> {
    let listener = TcpListener::bind("127.0.0.1:50052").await?;
    println!("Server listening on 127.0.0.1:50052");

    loop {
        let (mut stream, addr) = listener.accept().await?;
        println!("New connection from {}", addr);
        // Отправляем HTTP/2 preface
        stream.writable().await?;
        if let Err(e) = stream.try_write(b"PRI * HTTP/2.0\r\n\r\nSM\r\n\r\n") {
            eprintln!("Failed to send HTTP/2 preface to {}: {}", addr, e);
            continue;
        }
        println!("Sent HTTP/2 preface to {}", addr);

        // Создаем новую задачу для обработки соединения
        tokio::spawn(async move {
            // Даем клиенту время отправить данные
            tokio::time::sleep(Duration::from_millis(3000)).await;
            println!("Attempting to read Data frame from {}", addr);
            match read_frames(&mut stream).await {
                Ok(Frame::Data { payload, .. }) => {
                    println!("Received: {} from {}", String::from_utf8_lossy(&payload), addr);
                    // Отвечаем сообщением "world"
                    let mut response = BytesMut::with_capacity(32);
                    encode_message(b"world", &mut response);
                    let frame = Frame::Data {
                        stream_id: 1,
                        end_stream: true,
                        payload: response.freeze(),
                    };
                    if let Err(e) = send_frame(&mut stream, frame).await {
                        eprintln!("Failed to send response to {}: {}", addr, e);
                    } else {
                        println!("Sent 'world' to {}", addr);
                    }
                }
                Ok(_) => {
                    eprintln!("Received unexpected frame type from {}", addr);
                }
                Err(e) => {
                    eprintln!("Failed to read Data frame from {}: {}", addr, e);
                }
            }
        });
    }
}