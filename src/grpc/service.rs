use anyhow::Result;
use bytes::BytesMut;
use tokio::io::AsyncReadExt;
use tokio::net::TcpStream;

use crate::h2::{frame::Frame, sender::send_frame};
use super::codec::{decode_message, encode_message};

pub async fn handle_echo(stream: &mut TcpStream, stream_id: u32) -> Result<()> {
    let mut buf = BytesMut::with_capacity(1024);
    loop {
        let n = stream.read_buf(&mut buf).await?;
        if n == 0 { break; }
        if let Some(msg) = decode_message(&mut buf) {
            // echo back
            let mut payload = BytesMut::with_capacity(128);
            encode_message(&msg, &mut payload);
            let frame = Frame::Data { stream_id, end_stream: true, payload: payload.freeze() };
            send_frame(stream, frame).await?;
        }
    }
    Ok(())
}