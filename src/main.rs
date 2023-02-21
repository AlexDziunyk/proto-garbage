use anyhow::Result;
use protocol::Protocol;

mod protocol;

fn main() -> Result<()> {
    let protocol = Protocol::from_file("resources/protocol.toml")?;

    println!("{protocol:?}");
    Ok(())
}
