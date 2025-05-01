use bytes::{Buf, BufMut, Bytes, BytesMut};

pub fn encode_message(msg: &[u8], dst: &mut BytesMut) {
    dst.put_u8(0);                     // flags
    dst.put_u32(msg.len() as u32);     // length
    dst.extend_from_slice(msg);
}

pub fn decode_message(src: &mut BytesMut) -> Option<Bytes> {
    if src.len() < 5 { return None; }
    let len = (&src[1..5]).get_u32();
    if src.len() < 5 + len as usize { return None; }
    src.advance(5);
    Some(src.split_to(len as usize).freeze())
}