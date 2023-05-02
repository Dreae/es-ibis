use tokio::{sync::mpsc::{Sender, Receiver}, spawn};
use super::socket::EVEProtoSocket;

pub struct EVEClient {
    socket: EVEProtoSocket,
    server_commands: Receiver<u8>,
    client_commands: Sender<u8>
}

impl EVEClient {
    pub fn new(socket: EVEProtoSocket, command_channels: (Receiver<u8>, Sender<u8>)) -> Self {
        let (server_commands, client_commands) = command_channels;
        Self {
            socket,
            server_commands,
            client_commands
        }
    }

    pub async fn run(&mut self) {

    }

    pub fn spawn(mut self) {
        spawn(async move {
            self.run().await;
        });
    }
}