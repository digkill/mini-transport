use bytes::{Buf, BufMut, Bytes, BytesMut};

#[derive(Debug, Clone)]
pub enum Frame {
    Data { stream_id: u32, end_stream: bool, payload: Bytes },
    Headers { stream_id: u32, end_stream: bool, header_block: Bytes },
    Settings(Vec<(u16, u32)>),
}

impl Frame {
    pub fn encode(&self, dst: &mut BytesMut) {
        match self {
            Self::Data { stream_id, end_stream, payload } => {
                let len = payload.len() as u64;
                dst.put_u64(len);          // length
                dst.put_u8(0x0);           // type DATA
                dst.put_u8(if *end_stream { 0x1 } else { 0x0 });
                dst.put_u32(*stream_id & 0x7fff_ffff);
                dst.extend_from_slice(payload);
            }
            Self::Headers { stream_id, end_stream, header_block } => {
                let len = header_block.len() as u64;
                dst.put_u64(len);
                dst.put_u8(0x1);           // HEADERS
                dst.put_u8(if *end_stream { 0x1 } else { 0x4 }); // END_STREAM | END_HEADERS mock
                dst.put_u32(*stream_id & 0x7fff_ffff);
                dst.extend_from_slice(header_block);
            }
            Self::Settings(pairs) => {
                let len = (pairs.len() * 6) as u64;
                dst.put_u64(len);
                dst.put_u8(0x4); // SETTINGS
                dst.put_u8(0);
                dst.put_u32(0);
                for (id, val) in pairs {
                    dst.put_u16(*id);
                    dst.put_u32(*val);
                }
            }
        }
    }

    pub fn decode(src: &mut BytesMut) -> Option<Self> {
        if src.len() < 9 {
            return None;
        }
        let len = ((src[0] as u32) << 16) | ((src[1] as u32) << 8) | src[2] as u32;
        if src.len() < (9 + len as usize) {
            return None;
        }
        let ty = src[3];
        let flags = src[4];
        let stream_id = u32::from_be_bytes([src[5], src[6], src[7], src[8]]) & 0x7fff_ffff;
        src.advance(9);
        let payload = src.split_to(len as usize).freeze();
        Some(match ty {
            0x0 => Self::Data { stream_id, end_stream: flags & 0x1 != 0, payload },
            0x1 => Self::Headers { stream_id, end_stream: flags & 0x1 != 0, header_block: payload },
            0x4 => {
                let mut pairs = Vec::new();
                let mut p = payload.clone();
                while p.has_remaining() {
                    pairs.push((p.get_u16(), p.get_u32()));
                }
                Self::Settings(pairs)
            }
            _ => return None,
        })
    }
}