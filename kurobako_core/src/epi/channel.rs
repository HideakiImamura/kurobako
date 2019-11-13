//! The receiving and sending channels used to communicate with the external problems that support EPI.
use crate::{Error, Result};
use serde::{Deserialize, Serialize};
use serde_json;
use std::fmt;
use std::io::{BufRead, BufReader, BufWriter, Read, Write};
use std::marker::PhantomData;

/// Sending channel.
pub struct MessageSender<T, W: Write> {
    writer: BufWriter<W>,
    _message: PhantomData<T>,
}
impl<T, W> MessageSender<T, W>
where
    T: Serialize,
    W: Write,
{
    /// Makes a new `MessageSender` instance.
    pub fn new(writer: W) -> Self {
        Self {
            writer: BufWriter::new(writer),
            _message: PhantomData,
        }
    }

    /// Sends a message.
    pub fn send(&mut self, message: &T) -> Result<()> {
        track!(write!(self.writer, "kurobako:").map_err(Error::from))?;
        track!(serde_json::to_writer(&mut self.writer, message).map_err(Error::from))?;
        track!(writeln!(self.writer).map_err(Error::from))?;
        track!(self.writer.flush().map_err(Error::from))?;
        Ok(())
    }
}
impl<T, W: Write> fmt::Debug for MessageSender<T, W> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "MessageSender {{ .. }}")
    }
}

/// Receiving channel.
pub struct MessageReceiver<T, R: Read> {
    reader: BufReader<R>,
    _message: PhantomData<T>,
}
impl<T, R> MessageReceiver<T, R>
where
    T: for<'a> Deserialize<'a>,
    R: Read,
{
    /// Makes a new `MessageReceiver` instance.
    pub fn new(reader: R) -> Self {
        Self {
            reader: BufReader::new(reader),
            _message: PhantomData,
        }
    }

    /// Receives a message.
    pub fn recv(&mut self) -> Result<T> {
        let mut line = String::new();
        loop {
            track!(self.reader.read_line(&mut line).map_err(Error::from))?;
            if !line.starts_with("kurobako:") {
                eprintln!("{}", line);
                continue;
            }

            let message = track!(serde_json::from_str(&line).map_err(Error::from))?;
            return Ok(message);
        }
    }
}
impl<T, R: Read> fmt::Debug for MessageReceiver<T, R> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "MessageReceiver {{ .. }}")
    }
}
