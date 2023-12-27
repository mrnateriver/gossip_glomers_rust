mod echo;
mod server;

fn main() -> anyhow::Result<()> {
    let mut server = server::MaelstromServer::new(echo::EchoMessageHandler::new());

    let stdin = std::io::stdin().lines();

    for line in stdin {
        let resp =
            match serde_json::from_str::<server::Message<echo::EchoMessageContent>>(&line.unwrap())
            {
                Ok(deser) => {
                    if let Some(response) = server.handle(deser)? {
                        response
                    } else {
                        continue;
                    }
                }
                Err(e) => {
                    server.create_error(12, &format!("failed to deserialize input: {:#?}", e))
                }
            };
        let ser = serde_json::to_string(&resp).unwrap();
        println!("{}", ser);
    }

    Ok(())
}
