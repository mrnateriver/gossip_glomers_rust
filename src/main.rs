use echo::EchoMessageHandler;
use generate_id::GenerateIdMessageHandler;
use serde_json::{de::StrRead, Deserializer};

mod bus;
mod echo;
mod errors;
mod generate_id;
mod server;

fn main() -> anyhow::Result<()> {
    let mut server = server::MaelstromServer::new();
    server.register_handler::<EchoMessageHandler>();
    server.register_handler::<GenerateIdMessageHandler>();

    let stdin = std::io::stdin().lines();

    for line in stdin {
        let line = line.unwrap();
        let mut de = Deserializer::new(StrRead::new(line.as_ref()));

        server.input(&mut de);

        while let Some(resp) = server.output() {
            let ser = serde_json::to_string(&resp).unwrap();
            println!("{}", ser);
        }
    }

    Ok(())
}
