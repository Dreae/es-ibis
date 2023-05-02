use eve_proto::value::EVEValue;
use tokio::net::TcpStream;
use std::io::Result;

pub struct EVEProtoSocket {
    connection: TcpStream
}

impl EVEProtoSocket {
    pub fn new(connection: TcpStream) -> Self {
        Self {
            connection
        }
    }

    pub fn read_packet(&mut self) -> Result<Option<EVEValue>> {
        unimplemented!()
    }

    pub fn write_packet(&mut self) -> Result<()> {
        unimplemented!()
    }
}