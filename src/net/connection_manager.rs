use super::{EVEClient, socket::EVEProtoSocket};
use tokio::{sync::mpsc::{channel, Sender, Receiver}, spawn};

struct TrackedClient {
    server_commands: Sender<u8>,
    client_commands: Receiver<u8>
}

pub struct ClientConnectionManager {
    connections: Vec<TrackedClient>
}

impl ClientConnectionManager {
    pub fn new() -> Self {
        Self {
            connections: vec![]
        }
    }

    pub fn track(&mut self, socket: EVEProtoSocket) {
        let (server_cmd_s, server_cmd_r) = channel(12);
        let (client_cmd_s, client_cmd_r) = channel(48);

        let client = EVEClient::new(socket, (server_cmd_r, client_cmd_s));
        self.connections.push(TrackedClient {
            server_commands: server_cmd_s,
            client_commands: client_cmd_r
        });

        client.spawn();
    }
}