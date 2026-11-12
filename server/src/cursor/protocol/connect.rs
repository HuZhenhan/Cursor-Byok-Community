//! Encodes and decodes Connect protocol frames.
use std::io::Read;

use bytes::{BufMut, Bytes, BytesMut};
use prost::Message;
use serde::Serialize;

use crate::{Error, Result};

pub const COMPRESSED_FLAG: u8 = 0x01;
pub const END_STREAM_FLAG: u8 = 0x02;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ConnectCode {
    Canceled,
    InvalidArgument,
    NotFound,
    Unavailable,
    Internal,
}

impl ConnectCode {
    fn as_str(self) -> &'static str {
        match self {
            Self::Canceled => "canceled",
            Self::InvalidArgument => "invalid_argument",
            Self::NotFound => "not_found",
            Self::Unavailable => "unavailable",
            Self::Internal => "internal",
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct ConnectErrorDetail {
    #[serde(rename = "type")]
    pub type_name: String,
    pub value: String,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ConnectStreamError {
    pub code: ConnectCode,
    pub message: String,
    pub details: Vec<ConnectErrorDetail>,
}

#[derive(Serialize)]
struct EndStreamResponse<'a> {
    error: WireError<'a>,
}

#[derive(Serialize)]
struct WireError<'a> {
    code: &'static str,
    #[serde(skip_serializing_if = "str::is_empty")]
    message: &'a str,
    #[serde(skip_serializing_if = "details_are_empty")]
    details: &'a [ConnectErrorDetail],
}

fn details_are_empty(details: &&[ConnectErrorDetail]) -> bool {
    details.is_empty()
}

pub fn encode_message<M: Message>(message: &M) -> Result<Bytes> {
    let len = message.encoded_len();
    let mut output = BytesMut::with_capacity(5 + len);
    output.put_u8(0);
    output.put_u32(len as u32);
    message.encode(&mut output)?;
    Ok(output.freeze())
}

pub fn encode_end_stream() -> Bytes {
    encode_end_stream_payload(b"{}")
}

pub fn encode_error_end_stream(error: &ConnectStreamError) -> Result<Bytes> {
    let payload = serde_json::to_vec(&EndStreamResponse {
        error: WireError {
            code: error.code.as_str(),
            message: &error.message,
            details: &error.details,
        },
    })?;
    Ok(encode_end_stream_payload(&payload))
}

fn encode_end_stream_payload(payload: &[u8]) -> Bytes {
    let mut output = BytesMut::with_capacity(5 + payload.len());
    output.put_u8(END_STREAM_FLAG);
    output.put_u32(payload.len() as u32);
    output.extend_from_slice(payload);
    output.freeze()
}

pub fn decode_unary<M: Message + Default>(body: &[u8]) -> Result<M> {
    if body.len() >= 5 {
        let flags = body[0];
        let length = u32::from_be_bytes([body[1], body[2], body[3], body[4]]) as usize;
        if flags & END_STREAM_FLAG == 0 && length == body.len() - 5 {
            return Ok(M::decode(&body[5..])?);
        }
    }
    Ok(M::decode(body)?)
}

pub fn decode_frames(mut body: &[u8]) -> Result<Vec<(u8, Bytes)>> {
    let mut frames = Vec::new();
    while !body.is_empty() {
        if body.len() < 5 {
            return Err(Error::Protocol("truncated Connect envelope".into()));
        }
        let flags = body[0];
        let length = u32::from_be_bytes([body[1], body[2], body[3], body[4]]) as usize;
        body = &body[5..];
        if body.len() < length {
            return Err(Error::Protocol("truncated Connect payload".into()));
        }
        frames.push((flags, Bytes::copy_from_slice(&body[..length])));
        body = &body[length..];
    }
    Ok(frames)
}

/// 增量解析 Connect envelope；用于 HTTP/2 双向流里逐个到达的消息。
pub struct FrameDecoder {
    buffer: BytesMut,
}

impl FrameDecoder {
    pub fn new() -> Self {
        Self {
            buffer: BytesMut::new(),
        }
    }

    pub fn push(&mut self, chunk: &[u8]) {
        self.buffer.extend_from_slice(chunk);
    }

    pub fn pop(&mut self) -> Result<Option<(u8, Bytes)>> {
        if self.buffer.len() < 5 {
            return Ok(None);
        }
        let length = u32::from_be_bytes([
            self.buffer[1],
            self.buffer[2],
            self.buffer[3],
            self.buffer[4],
        ]) as usize;
        if self.buffer.len() < 5 + length {
            return Ok(None);
        }
        let mut frame = self.buffer.split_to(5 + length);
        let header = frame.split_to(5);
        let payload = frame.freeze();
        Ok(Some((header[0], payload)))
    }

    pub fn take_buffered(&mut self) -> Bytes {
        std::mem::take(&mut self.buffer).freeze()
    }
}

/// 取出 envelope 里的消息体；带压缩标志时按 gzip 解压。
pub fn message_payload(flags: u8, payload: Bytes) -> Result<Bytes> {
    if flags & COMPRESSED_FLAG == 0 {
        return Ok(payload);
    }
    let mut decoder = flate2::read::GzDecoder::new(payload.as_ref());
    let mut decoded = Vec::new();
    decoder
        .read_to_end(&mut decoded)
        .map_err(|error| Error::Protocol(format!("cannot decompress Connect message: {error}")))?;
    Ok(Bytes::from(decoded))
}

#[cfg(test)]
mod tests {
    use std::io::Write;

    use super::*;

    #[test]
    fn frame_decoder_handles_split_chunks() {
        let body = [&[0u8, 0, 0, 0, 3][..], b"abc", &[0u8, 0, 0, 0, 2][..], b"de"].concat();
        let mut decoder = FrameDecoder::new();
        decoder.push(&body[..4]);
        assert!(decoder.pop().unwrap().is_none());
        decoder.push(&body[4..9]);
        assert_eq!(decoder.pop().unwrap().unwrap().1.as_ref(), b"abc");
        decoder.push(&body[9..]);
        assert_eq!(decoder.pop().unwrap().unwrap().1.as_ref(), b"de");
        assert!(decoder.pop().unwrap().is_none());
    }

    #[test]
    fn compressed_messages_are_gzip_decoded() {
        let mut encoder = flate2::write::GzEncoder::new(Vec::new(), flate2::Compression::default());
        encoder.write_all(b"hello").unwrap();
        let compressed = encoder.finish().unwrap();
        let payload = message_payload(COMPRESSED_FLAG, Bytes::from(compressed)).unwrap();
        assert_eq!(payload.as_ref(), b"hello");
    }
}

