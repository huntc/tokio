use bytes::{Buf, BufMut, Bytes, BytesMut};

use crate::codec::{Decoder, Encoder};
use std::{fmt, io, usize};

/// Consistent Overhead Byte Stuffing (COBS) is an algorithm for encoding data bytes that results in efficient,
/// reliable, unambiguous packet framing regardless of packet content, thus making it easy for receiving applications
/// to recover from malformed packets. It employs a particular byte value, typically zero, to serve as a packet delimiter
/// (a special value that indicates the boundary between packets). When zero is used as a delimiter, the algorithm
/// replaces each zero data byte with a non-zero value so that no zero data bytes will appear in the packet and thus be
/// misinterpreted as packet boundaries.
///
/// Documentation credit: [Wikipedia](https://en.wikipedia.org/wiki/Consistent_Overhead_Byte_Stuffing)
#[derive(Debug)]
pub struct CobsCodec {
    // The delimiter byte to use. Usually a 0 with COBS.
    delimiter: u8,

    // The maximum length for a given read.
    max_length: usize,

    // Are we currently discarding the remainder of a bytes which was over
    // the length limit?
    is_discarding: bool,
}

impl CobsCodec {
    /// Provide a new COBS codec that scans up to a limited number of bytes in total.
    /// A 0 is used as the delimiter,
    pub fn new(max_length: usize) -> Self {
        Self::new_with_delimiter(0, max_length)
    }

    /// Provide a new COBS codec with a specific delimiter that scans up to a limited number
    /// of bytes in total.
    pub fn new_with_delimiter(delimiter: u8, max_length: usize) -> Self {
        Self {
            delimiter,
            max_length,
            is_discarding: false,
        }
    }
}

const CHUNK_LEN: usize = 254;
const MAX_BYTE_OVERHEAD: usize = 2;

impl Decoder for CobsCodec {
    type Item = BytesMut;
    type Error = CobsCodecError;

    fn decode(&mut self, src: &mut BytesMut) -> Result<Option<Self::Item>, Self::Error> {
        todo!()
    }
}

impl Encoder<Bytes> for CobsCodec {
    type Error = CobsCodecError;

    fn encode(&mut self, src: Bytes, dst: &mut BytesMut) -> Result<(), Self::Error> {
        let encoded_len = (src.len() / CHUNK_LEN) * (CHUNK_LEN + MAX_BYTE_OVERHEAD);
        let encoded_remaining_len = src.len() % CHUNK_LEN;
        let encoded_len = if encoded_remaining_len > 0 {
            encoded_len + encoded_remaining_len + MAX_BYTE_OVERHEAD
        } else {
            encoded_len
        };
        dst.reserve(encoded_len);
        for (i, byte) in src.iter().enumerate() {
            if i % CHUNK_LEN == 0 {
                dst.put_u8(0);
            }
            dst.put_u8(*byte);
        }
        dst.put_u8(0);
        let mut distance = 0;
        for byte in dst.iter_mut().rev() {
            if *byte == self.delimiter {
                if distance > 0 {
                    *byte = distance;
                }
                distance = 1;
            } else {
                distance += 1;
            }
        }
        Ok(())
    }
}

/// An error occurred while encoding or decoding.
#[derive(Debug)]
pub enum CobsCodecError {
    /// The maximum length was exceeded.
    MaxLengthExceeded,
    /// An IO error occurred.
    Io(io::Error),
}

impl fmt::Display for CobsCodecError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            CobsCodecError::MaxLengthExceeded => write!(f, "max length exceeded"),
            CobsCodecError::Io(e) => write!(f, "{}", e),
        }
    }
}

impl From<io::Error> for CobsCodecError {
    fn from(e: io::Error) -> CobsCodecError {
        CobsCodecError::Io(e)
    }
}

impl std::error::Error for CobsCodecError {}
