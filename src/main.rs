use sourcon::client::Client;
use std::error::Error;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let host = "dev.viora.sh:27016";
    let mut client = Client::connect(host, "poopxd").await?;
    let status = client.command("status").await?;
    println!("{}", status.body());
    Ok(())
}
