use std::time::SystemTime;
use fern::colors::{Color, ColoredLevelConfig};

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
        .level(log::LevelFilter::Debug)
        .chain(std::io::stdout())
        .apply()?;
    Ok(())
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    setup_logger()?;

    log::info!("Hello, world!");

    Ok(())
}