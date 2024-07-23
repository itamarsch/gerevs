use std::io::{self, Cursor};

use tokio::io::{AsyncReadExt, AsyncSeekExt};

use crate::protocol::{SocksSocketAddr, RESERVED_16};

#[derive(Debug)]
pub struct UdpMessage<'a> {
    pub fragment_number: u8,
    pub dst: SocksSocketAddr,
    pub data: &'a [u8],
}
impl<'a> UdpMessage<'a> {
    pub fn as_bytes(&self) -> Vec<u8> {
        let mut res: Vec<u8> = Vec::with_capacity(self.data.len() + 32);
        res.extend_from_slice(&RESERVED_16.to_be_bytes());
        res.push(self.fragment_number);
        res.extend(self.dst.to_bytes());
        res.extend_from_slice(self.data);
        res
    }

    pub async fn parse(buf: &'a [u8]) -> io::Result<Self> {
        let mut cursor = Cursor::new(buf);

        let reserved = cursor.read_u16().await?;
        if reserved != RESERVED_16 {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                "Reserved bytes need to be zero but weren't",
            ));
        }

        let fragment_number = cursor.read_u8().await?;
        let dst = SocksSocketAddr::read(&mut cursor).await?;

        let current_pos = cursor.stream_position().await? as usize;
        let data = &buf[current_pos..];
        Ok(UdpMessage {
            fragment_number,
            dst,
            data,
        })
    }
}
