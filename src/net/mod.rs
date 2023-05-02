mod client;
mod server;
mod socket;
mod connection_manager;

pub use server::EVEServer;
pub use client::EVEClient;
pub use connection_manager::ClientConnectionManager;