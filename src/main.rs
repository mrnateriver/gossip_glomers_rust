#![feature(try_trait_v2)]

use messages::{EchoMessageHandler, GenerateIdMessageHandler};
use serde_json::{de::StrRead, Deserializer};
use server::MaelstromService;

mod messages;
mod protocol;
mod server;

fn main() -> anyhow::Result<()> {
    let mut server = MaelstromService::new();
    server.register_handler::<EchoMessageHandler>();
    server.register_handler::<GenerateIdMessageHandler>();

    let stdin = std::io::stdin().lines();

    for line in stdin {
        let line = line.unwrap();
        let mut de = Deserializer::new(StrRead::new(line.as_ref()));

        for resp in server.input(&mut de) {
            let ser = serde_json::to_string(&resp).unwrap();
            println!("{}", ser);
        }
    }

    Ok(())
}
