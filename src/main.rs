use std::time::SystemTime;
use fern::colors::{Color, ColoredLevelConfig};
use tokio::net::TcpListener;

use crate::net::EVEServer;

mod net;

const VERSION: &'static str = "v0.0.1";

fn setup_logger() -> Result<(), fern::InitError> {
    let colors = ColoredLevelConfig::default()
        .info(Color::Blue);
    fern::Dispatch::new()
        .format(move|out, message, record| {
            out.finish(format_args!(
                "[{} {} {}] {}",
                humantime::format_rfc3339_seconds(SystemTime::now()),
                colors.color(record.level()),
                record.target(),
                message
            ))
        })
        .level(log::LevelFilter::Trace)
        .chain(std::io::stdout())
        .apply()?;
    Ok(())
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    setup_logger()?;

    log::info!("es-ibis version {}", self::VERSION);
    let listener = TcpListener::bind("127.0.0.1:26000").await?;
    let mut server = EVEServer::new(listener);

    server.run().await;

    Ok(())
}