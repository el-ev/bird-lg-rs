use std::{
    pin::Pin,
    task::{Context as TaskContext, Poll},
};

use anyhow::{Context as _, bail};
use bytes::BytesMut;
use tokio::{
    io::{AsyncRead, AsyncReadExt, AsyncWrite, AsyncWriteExt},
    net::UnixStream,
};
use tokio_stream::Stream;
use tokio_util::codec::{Decoder, Framed};

pub async fn connect(socket_path: &str) -> anyhow::Result<UnixStream> {
    let mut stream = UnixStream::connect(socket_path)
        .await
        .with_context(|| format!("Failed to connect to bird socket {}", socket_path))?;

    let mut buffer = [0; 1024];
    let n = stream
        .read(&mut buffer)
        .await
        .context("Failed to read greeting from bird socket")?;

    if !buffer[..n].starts_with(b"0001") {
        bail!("Unexpected birdc response: {:?}", &buffer[..n]);
    }

    stream
        .write_all(b"restrict\n")
        .await
        .context("Failed to enable restrict mode on bird socket")?;

    buffer.fill(0);
    let n = stream
        .read(&mut buffer)
        .await
        .context("Failed to confirm restrict mode on bird socket")?;

    if !buffer[..n].starts_with(b"0016") {
        bail!("Unable to set birdc restrict mode: {:?}", &buffer[..n]);
    }

    Ok(stream)
}

#[derive(Default)]
pub struct BirdDecoder {
    last_type: u8,
    current_message: String,
}

pub struct BirdLine {
    content: String,
    is_last: bool,
}

impl Decoder for BirdDecoder {
    type Item = BirdLine;
    type Error = std::io::Error;

    fn decode(&mut self, src: &mut BytesMut) -> Result<Option<Self::Item>, Self::Error> {
        loop {
            if let Some(offset) = src.iter().position(|&b| b == b'\n') {
                let line_len = offset + 1;
                let line_bytes = src.split_to(line_len);
                let line = &line_bytes[..offset];

                if line.len() >= 4 && line[0..4].iter().all(|&b| b.is_ascii_digit()) {
                    self.last_type = line[0];
                    if line.len() >= 5 {
                        self.current_message
                            .push_str(&String::from_utf8_lossy(&line[5..]));
                    }
                } else {
                    self.current_message
                        .push_str(&String::from_utf8_lossy(line));
                }
                self.current_message.push('\n');

                let is_last = b"089".contains(&self.last_type);

                if is_last {
                    let content = std::mem::take(&mut self.current_message);
                    return Ok(Some(BirdLine { content, is_last }));
                }
            } else {
                return Ok(None);
            }
        }
    }
}

pub struct BirdStream<T> {
    pub inner: Framed<T, BirdDecoder>,
    pub done: bool,
}

impl<T> Stream for BirdStream<T>
where
    T: AsyncRead + AsyncWrite + Unpin,
{
    type Item = Result<String, std::io::Error>;

    fn poll_next(mut self: Pin<&mut Self>, cx: &mut TaskContext<'_>) -> Poll<Option<Self::Item>> {
        if self.done {
            return Poll::Ready(None);
        }

        match std::task::ready!(Pin::new(&mut self.inner).poll_next(cx)) {
            Some(Ok(line)) => {
                if line.is_last {
                    self.done = true;
                }
                Poll::Ready(Some(Ok(line.content)))
            }
            Some(Err(e)) => Poll::Ready(Some(Err(e))),
            None => Poll::Ready(None),
        }
    }
}
