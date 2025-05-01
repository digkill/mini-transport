use anyhow::Result;
use bytes::BytesMut;
use tokio::{io::AsyncReadExt, net::TcpStream};

use super::frame::Frame;

pub async fn read_frames(stream: &mut TcpStream) -> Result<Frame> {
    let mut buf = BytesMut::with_capacity(16 * 1024);
    loop {
        // try parse existing data first
        if let Some(frame) = Frame::decode(&mut buf) {
            return Ok(frame);
        }
        // need more data
        let n = stream.read_buf(&mut buf).await?;
        if n == 0 {
            anyhow::bail!("connection closed");
        }
    }
}