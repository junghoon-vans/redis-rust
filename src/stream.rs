use tokio::net::TcpStream;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use bytes::BytesMut;

#[derive(Clone, Debug)]
pub enum Value {
    SimpleString(String),
    BulkString(String),
    Array(Vec<Value>),
    Null,
}

impl Value {
    pub fn to_string(self) -> String {
        match self {
            Value::SimpleString(s) => format!("+{}\r\n", s),
            Value::BulkString(s) => format!("${}\r\n{}\r\n", s.chars().count(), s),
            Value::Null => "$-1\r\n".to_string(),
            _ => panic!("Not implemented"),
        }
    }
}

pub struct StreamHandler {
    stream: TcpStream,
    buffer: BytesMut,
}

impl StreamHandler {
    pub fn new(stream: TcpStream) -> Self {
        Self {
            stream,
            buffer: BytesMut::with_capacity(512),
        }
    }

    pub async fn read_request(&mut self) -> Result<Option<Value>, anyhow::Error> {
        let bytes_read = self.stream.read_buf(&mut self.buffer).await?;

        if bytes_read == 0 {
            return Ok(None);
        }

        let v = parse_message(self.buffer.split())?;
        Ok(Some(v.0))
    }

    pub async fn write_response(&mut self, value: Value) -> Result<(), anyhow::Error> {
        self.stream.write(value.to_string().as_bytes()).await?;

        Ok(())
    }
}

fn parse_message(buffer: BytesMut) -> Result<(Value, usize), anyhow::Error> {
    match buffer[0] as char {
        '+' => parse_simple_string(buffer),
        '*' => parse_array(buffer),
        '$' => parse_bulk_string(buffer),
        _ => Err(anyhow::anyhow!("Invalid message format {:?}", buffer)),
    }
}

fn parse_simple_string(buffer: BytesMut) -> Result<(Value, usize), anyhow::Error> {
    if let Some((line, len)) = read_until_crlf(&buffer[1..]) {
        let string = String::from_utf8(line.to_vec()).unwrap();
        return Ok((Value::SimpleString(string), len + 1));
    } else {
        Err(anyhow::anyhow!("Invalid string {:?}", buffer))
    }
}

fn parse_array(buffer: BytesMut) -> Result<(Value, usize), anyhow::Error> {
    let (array_len, mut bytes_consumed)
        = if let Some((line, len)) = read_until_crlf(&buffer[1..]) {
        let array_len = parse_int(line)?;
        (array_len, len + 1)
    } else {
        return Err(anyhow::anyhow!("Invalid array format {:?}", buffer));
    };

    let mut items = vec![];
    for _ in 0..array_len {
        let (item, len) = parse_message(BytesMut::from(&buffer[bytes_consumed..]))?;
        items.push(item);
        bytes_consumed += len;
    }

    return Ok((Value::Array(items), bytes_consumed));
}

fn parse_bulk_string(buffer: BytesMut) -> Result<(Value, usize), anyhow::Error> {
    let (bulk_str_len, bytes_consumed)
        = if let Some((line, len)) = read_until_crlf(&buffer[1..]) {
        let bulk_str_len = parse_int(line)?;
        (bulk_str_len, len + 1)
    } else {
        return Err(anyhow::anyhow!("Invalid array format {:?}", buffer));
    };

    let end_of_bulk_str = bytes_consumed + bulk_str_len as usize;
    let total_parsed = end_of_bulk_str + 2;

    return Ok((Value::BulkString(String::from_utf8(buffer[bytes_consumed..end_of_bulk_str].to_vec())?), total_parsed));
}

fn read_until_crlf(buffer: &[u8]) -> Option<(&[u8], usize)> {
    for i in 1..buffer.len() {
        if buffer[i - 1] == b'\r' && buffer[i] == b'\n' {
            return Some((&buffer[0..(i - 1)], i + 1));
        }
    }
    return None;
}

fn parse_int(buffer: &[u8]) -> Result<i64, anyhow::Error> {
    String::from_utf8(buffer.to_vec())?.parse::<i64>().map_err(|e| anyhow::anyhow!(e))
}
