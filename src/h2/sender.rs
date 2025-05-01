use anyhow::Result;
use bytes::BytesMut;
use tokio::{io::AsyncWriteExt, net::TcpStream};

use super::frame::Frame;

pub async fn send_frame(stream: &mut TcpStream, frame: Frame) -> Result<()> {
    let mut buf = BytesMut::with_capacity(16 * 1024);
    frame.encode(&mut buf);
    stream.write_all(&buf).await?;
    Ok(())
}