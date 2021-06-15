use futures::StreamExt;
use tokio_dlt::{DltTcpClient, DltTcpClientOptions, Error};

#[tokio::main]
async fn main() -> Result<(), Error> {
    let opts = DltTcpClientOptions {
        host: "127.0.0.1".to_string(),
        port: 3490,
    };
    let dlt_tcp_client = DltTcpClient::connect(&opts).await?;
    let mut s = dlt_tcp_client.read();

    while let Some(result) = s.next().await {
        println!("Message: {:?}", result);
    }

    Ok(())
}
