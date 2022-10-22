use std::{io::{Read, self, Write, Error, ErrorKind}, fs::File, path::Path};

use byteorder::{WriteBytesExt, BigEndian, ReadBytesExt};
use serde_json::json;

use crate::inverted_index::QueryResult;

pub struct Message {
    kind: u8,
    len: u64,
    content: Option<MessageContent>
}

pub enum MessageContent {
    String(String),
    Stream(StreamContent)
}

pub struct StreamContent {
    stream: Box<dyn Read>,
    len: u64,
}

impl StreamContent {
    fn from_file(f: File) -> io::Result<Self> {
        let len = f.metadata()?.len();
        Ok(StreamContent {
            len, 
            stream: Box::new(f)
        })
    }
}

impl std::fmt::Debug for MessageContent {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::String(arg0) => f.debug_tuple("String").field(arg0).finish(),
            Self::Stream(_) => f.debug_tuple("Stream").finish(),
        }
    }
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
            match content {
                MessageContent::String(s) => {
                    let mut s_bytes = s.as_bytes();
                    while !s_bytes.is_empty() {
                        let written = stream.write(&s_bytes)?;
                        s_bytes = &s_bytes[written..];
                    }
                },
                MessageContent::Stream(in_stream) => {
                    io::copy(&mut in_stream.stream, stream)?;
                },
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
            Some(MessageContent::String(s))
        } else {
            None
        };

        Ok(Self { kind, len, content })
    }

    pub fn from_string<'a>(kind: u8, s: String) -> Self {
        Message { 
            kind, 
            len: s.len() as u64, 
            content: Some(MessageContent::String(s))
        }
    }

    pub fn from_stream_content(kind :u8, stream_content: StreamContent) -> Self {
        Message {
            kind,
            len: stream_content.len,
            content: Some(MessageContent::Stream(stream_content))
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
    FileResult(MessageContent)
}

impl Response {
    pub fn from_file_path(s: &String) -> io::Result<Self> {
        let path = Path::new(s);
        if !path.exists() {
            return Ok(Self::Error("file does not exist".to_owned()));
        } else if !path.is_file() {
            return Ok(Self::Error("path is not a file".to_owned()));
        }
        
        let f = File::open(path)?;
        let stream_content = StreamContent::from_file(f)?;
        Ok(Self::FileResult(MessageContent::Stream(stream_content)))
    }
}

impl IntoMessage for Response {
    fn into_message(self) -> Message {
        match self {
            Self::Pong => Message::empty(0),
            Self::Error(s) => Message::from_string(1, s),
            Self::QueryResult(v) => 
                Message::from_string(2, json!(v).to_string()),
            Self::FileResult(content) => match content {
                MessageContent::String(s) => Message::from_string(3, s),
                MessageContent::Stream(stream_content) =>
                    Message::from_stream_content(3, stream_content),
            },
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
            3 => Self::FileResult(MessageContent::String(
                requires_payload(content, "FileResult")?)),
            x => return Err(Error::new(ErrorKind::InvalidInput, 
                format!("response kind {} does not exist", x)))
        };
        Ok(response)
    }
}

fn requires_payload(content: Option<MessageContent>, message_kind: &str) -> io::Result<String> {
    let content = content.ok_or(Error::new(ErrorKind::InvalidInput, 
        format!("{} requires a payload", message_kind)))?;
    if let MessageContent::String(s) = content {
        Ok(s)
    } else {
        Err(Error::new(ErrorKind::Unsupported, 
            format!("messages with a stream payload are not supported for reading")))
    }
}
