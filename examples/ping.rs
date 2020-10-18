use worm::*;

#[tokio::main]
pub async fn main() -> Result<(), Error> {
    let mut client = Client::new("127.0.0.1:8080", Some(("test", "test"))).await?;

    let res = client.command(&["ping"]).await?;
    println!("{:?}", res);

    Ok(())
}
