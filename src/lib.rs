use async_stream::stream;
use dlt_core::{
    dlt,
    parse::{dlt_message, DltParseError, ParsedMessage},
};
use futures::Stream;
use std::{
    io,
    pin::Pin,
    task::{Context, Poll},
};
use thiserror::Error;
use tokio::{
    io::{AsyncRead, AsyncWrite, ReadBuf},
    net::TcpStream,
};

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

pub struct DltTcpClient {}

impl DltTcpClient {
    pub async fn connect_stream(opts: &DltTcpClientOptions) -> Result<DltStream, Error> {
        let tcp = TcpStream::connect((&*opts.host, opts.port)).await?;
        Ok(DltStream { tcp })
    }

    pub async fn read(
        opts: &DltTcpClientOptions,
    ) -> Result<impl Stream<Item = Result<dlt::Message, Error>>, Error> {
        let stream = TcpStream::connect((&*opts.host, opts.port)).await?;
        stream.readable().await?;

        let mut buf = [0; 4096];

        let s = stream! {
            loop {
                match stream.try_read(&mut buf) {
                    Ok(0) => break,
                    Ok(n) => {
                        println!("read {} bytes", n);
                        match dlt_message(&buf, None, false) {
                            Err(e) => yield Err(e.into()),
                            Ok((_, ParsedMessage::Item(message))) => yield Ok(message),
                            Ok((_, ParsedMessage::Invalid)) => yield Err(Error::DltInvalidMessage),
                            Ok((_, ParsedMessage::FilteredOut(_))) => continue,
                        }
                    }
                    Err(ref e) if e.kind() == io::ErrorKind::WouldBlock => {
                        continue;
                    }
                    Err(e) => {
                        yield Err(e.into());
                    }
                }
            }
        };

        Ok(s)
    }
}

pub struct DltStream {
    tcp: TcpStream,
}

impl AsyncRead for DltStream {
    fn poll_read(
        self: Pin<&mut Self>,
        cx: &mut Context,
        buf: &mut ReadBuf,
    ) -> Poll<std::io::Result<()>> {
        match Pin::get_mut(self) {
            DltStream { tcp } => Pin::new(tcp).poll_read(cx, buf),
        }
    }
}

impl AsyncWrite for DltStream {
    fn poll_write(
        self: Pin<&mut Self>,
        cx: &mut Context,
        buf: &[u8],
    ) -> Poll<std::result::Result<usize, tokio::io::Error>> {
        match Pin::get_mut(self) {
            DltStream { tcp } => Pin::new(tcp).poll_write(cx, buf),
        }
    }

    fn poll_flush(
        self: Pin<&mut Self>,
        cx: &mut Context,
    ) -> Poll<std::result::Result<(), tokio::io::Error>> {
        match Pin::get_mut(self) {
            DltStream { tcp } => Pin::new(tcp).poll_flush(cx),
        }
    }

    fn poll_shutdown(
        self: Pin<&mut Self>,
        cx: &mut Context,
    ) -> Poll<std::result::Result<(), tokio::io::Error>> {
        match Pin::get_mut(self) {
            DltStream { tcp } => Pin::new(tcp).poll_shutdown(cx),
        }
    }
}
