use std::{io::{Read, self, Write, Error, ErrorKind}, fs::File};

use byteorder::{WriteBytesExt, BigEndian, ReadBytesExt};
use serde_json::json;

use crate::inverted_index::QueryResult;

pub struct Message {
    kind: u8,
    len: u64,
    content: Option<String>
}

pub trait IntoMessage {
    fn into_message(self) -> Message;

    fn write(self, stream: &mut impl Write) -> io::Result<()>
        where Self : Sized 
    {
        let mut message = self.into_message();
        message.write(stream)
    }
}

pub trait FromMessage {
    fn from_message(message: Message) -> io::Result<Self>
        where Self: Sized;

    fn read(stream: &mut impl Read) -> io::Result<Self> 
        where Self: Sized
    {
        let message = Message::from_reader(stream)?;
        FromMessage::from_message(message)
    }
}

/// |message_kind(1B)|message_len(8B)|message({message_len}B)
impl Message {
    pub fn write(&mut self, stream: &mut impl Write) -> io::Result<()> {
        stream.write_u8(self.kind)?;
        stream.write_u64::<BigEndian>(self.len)?;

        if let Some(content) = &mut self.content {
            let mut content = content.as_bytes();
            while !content.is_empty() {
                let written = stream.write(&content)?;
                content = &content[written..];
            }
        }

        Ok(())
    }

    pub fn from_reader(stream: &mut impl Read) -> io::Result<Self> {
        let kind = stream.read_u8()?;
        let len = stream.read_u64::<BigEndian>()?;
        let content = if len > 0 {
            let mut buf = vec![0 as u8; len as usize];
            stream.read_exact(&mut buf[..])?;
            let s = String::from_utf8(buf).or(Err(Error::new(
                ErrorKind::InvalidData, "payload is not a valid UTF8 string")))?;
            Some(s)
        } else {
            None
        };

        Ok(Self { kind, len, content })
    }

    pub fn from_string<'a>(kind: u8, s: String) -> Self {
        Message { 
            kind, 
            len: s.len() as u64, 
            content: Some(s)
        }
    }

    pub fn empty(kind: u8) -> Self {
        Message { kind, len: 0, content: None }
    }
}

#[derive(Debug)]
pub enum Request {
    Ping,
    Query(String),
    QueryFile(String)
}

impl FromMessage for Request {
    fn from_message(message: Message) -> io::Result<Self> {
        let Message{ kind, len, content } = message;

        let request = match kind {
            0 => Self::Ping,
            1 => Self::Query(requires_payload(content, "Query")?),
            2 => Self::QueryFile(requires_payload(content, "QueryFile")?),
            x => return Err(Error::new(ErrorKind::InvalidInput, 
                format!("request kind {} does not exist", x)))
        };
        Ok(request)
    }
}

impl IntoMessage for Request {
    fn into_message(self) -> Message {
        match self {
            Request::Ping => Message::empty(0),
            Request::Query(s) => Message::from_string(1, s),
            Request::QueryFile(s) => Message::from_string(2, s),
        }
    }
}

#[derive(Debug)]
pub enum Response {
    Pong,
    Error(String),
    QueryResult(Vec<QueryResult>),
    FileResult(String)
}

impl IntoMessage for Response {
    fn into_message(self) -> Message {
        match self {
            Self::Pong => Message::empty(0),
            Self::Error(s) => Message::from_string(1, s),
            Self::QueryResult(v) => 
                Message::from_string(2, json!(v).to_string()),
            Self::FileResult(s) => Message::from_string(3, s),
        }
    }
}

impl FromMessage for Response {
    fn from_message(message: Message) -> io::Result<Self> {
        let Message{ kind, len, content } = message;

        let response =  match kind {
            0 => Self::Pong,
            1 => Self::Error(requires_payload(content, "Error")?),
            2 => {
                let content = requires_payload(content, "QueryResult")?;
                let v = serde_json::from_str::<Vec<QueryResult>>(&content)?;
                Self::QueryResult(v)
            },
            3 => Self::FileResult(requires_payload(content, "FileResult")?),
            x => return Err(Error::new(ErrorKind::InvalidInput, 
                format!("response kind {} does not exist", x)))
        };
        Ok(response)
    }
}

fn requires_payload(content: Option<String>, message_kind: &str) -> io::Result<String> {
    content.ok_or(Error::new(ErrorKind::InvalidInput, 
        format!("{} requires a payload", message_kind)))
}
