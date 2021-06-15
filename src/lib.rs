use bytes::{Buf, BufMut, BytesMut};
use dlt_core::{
    dlt,
    parse::{dlt_message, DltParseError, ParsedMessage},
};
use futures::{Sink, Stream};
use std::io;
use thiserror::Error;
use tokio::net::TcpStream;
use tokio_util::codec::{Decoder, Encoder, FramedRead, FramedWrite};

#[derive(Debug, Error)]
pub enum Error {
    #[error("DLT parsing error: {0:?}")]
    Dlt(#[from] DltParseError),
    #[error("Failed to parse message")]
    DltInvalidMessage,
    #[error("IO error: {0:?}")]
    Io(#[from] io::Error),
}

pub struct DltTcpClientOptions {
    pub host: String,
    pub port: u16,
}

pub struct DltTcpClient {
    tcp_stream: TcpStream,
}

impl DltTcpClient {
    pub async fn connect(opts: &DltTcpClientOptions) -> Result<Self, Error> {
        let tcp_stream = TcpStream::connect((&*opts.host, opts.port)).await?;
        Ok(DltTcpClient { tcp_stream })
    }

    pub fn read(self) -> impl Stream<Item = Result<dlt::Message, Error>> {
        FramedRead::new(self.tcp_stream, DltDecoder {})
    }

    pub fn write(self) -> impl Sink<dlt::Message, Error = std::io::Error> {
        FramedWrite::new(self.tcp_stream, DltEncoder {})
    }
}

struct DltDecoder {}

const MAX: usize = 8 * 1024 * 1024;

impl Decoder for DltDecoder {
    type Item = dlt::Message;
    type Error = Error;

    fn decode(&mut self, src: &mut BytesMut) -> Result<Option<Self::Item>, Self::Error> {
        // Header Type 1 byte + Message Counter 1 byte + Length 2 byte
        if src.len() < 4 {
            // Not enough data to read length marker.
            return Ok(None);
        }

        // Read length marker.
        let mut length_bytes = [0u8; 2];
        length_bytes.copy_from_slice(&src[2..=3]);
        let length = u16::from_be_bytes(length_bytes) as usize;

        // Check that the length is not too large to avoid a denial of
        // service attack where the server runs out of memory.
        if length > MAX {
            return Err(std::io::Error::new(
                std::io::ErrorKind::InvalidData,
                format!("Frame of length {} is too large.", length),
            )
            .into());
        }

        if src.len() < length {
            // The full string has not yet arrived.
            //
            // We reserve more space in the buffer. This is not strictly
            // necessary, but is a good idea performance-wise.
            src.reserve(length - src.len());

            // We inform the Framed that we need more bytes to form the next
            // frame.
            return Ok(None);
        }

        // Use advance to modify src such that it no longer contains
        // this frame.
        let data = src[..length].to_vec();
        src.advance(length);

        match dlt_message(&data, None, false) {
            Err(e) => Err(e.into()),
            Ok((_, ParsedMessage::Item(message))) => Ok(Some(message)),
            Ok((_, ParsedMessage::Invalid)) => Err(std::io::Error::new(
                std::io::ErrorKind::InvalidData,
                "Invalid DLT message data",
            )
            .into()),
            // This should not occur
            Ok((_, ParsedMessage::FilteredOut(_))) => {
                Err(std::io::Error::new(std::io::ErrorKind::InvalidData, "Filtered message").into())
            }
        }
    }
}

struct DltEncoder {}

impl Encoder<dlt::Message> for DltEncoder {
    type Error = io::Error;

    fn encode(&mut self, item: dlt::Message, dst: &mut BytesMut) -> io::Result<()> {
        let item_bytes = item.as_bytes();
        dst.reserve(item_bytes.len());
        dst.put(&*item_bytes);
        Ok(())
    }
}
