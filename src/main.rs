use log::{error, info, Level, Metadata, Record};
use sourcon::server;
use std::error::Error;
use tokio::signal;

struct SimpleLogger;

impl log::Log for SimpleLogger {
    fn enabled(&self, metadata: &Metadata) -> bool {
        metadata.level() <= Level::Info
    }

    fn log(&self, record: &Record) {
        if self.enabled(record.metadata()) {
            println!("{} - {}", record.level(), record.args());
        }
    }

    fn flush(&self) {}
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let _ = log::set_logger(&SimpleLogger).map(|()| log::set_max_level(log::LevelFilter::Info));

    let server = server::Server::start(|res| match res {
        Ok(packet) => info!("receive: {:?}", packet.body()),
        Err(err) => error!("error: {:?}", err),
    })
    .await?;

    tokio::select!(
        _ = server => {}
        _ = signal::ctrl_c() => {}
    );

    info!("bye");
    Ok(())
}
