use tokio::net::TcpListener;

use crate::net::socket::EVEProtoSocket;

use super::ClientConnectionManager;

pub struct EVEServer {
    listener: TcpListener,
    connection_manager: ClientConnectionManager
}

impl EVEServer {
    pub fn new(listener: TcpListener) -> Self {
        Self {
            listener,
            connection_manager: ClientConnectionManager::new()
        }
    }

    pub async fn run(&mut self) {
        loop {
            match self.listener.accept().await {
                Ok((socket, remote)) => {
                    log::trace!("Got connection from {}", remote);
                    let socket = EVEProtoSocket::new(socket);

                    self.connection_manager.track(socket);
                }
                Err(err) => {
                    log::error!("Accept failed: {:?}", err);
                }
            }
        }
    }
}