use futures::{StreamExt, pin_mut};
use tokio_dlt::{DltTcpClient, DltTcpClientOptions};


#[tokio::main]
async fn main() {
    let opts = DltTcpClientOptions { host: "127.0.0.1".to_string(), port: 3490};
    let s = DltTcpClient::read(&opts).await.unwrap();
    pin_mut!(s);
    while let Some(result) = s.next().await {
        println!("Message: {:?}", result );
    }
    // let message = s.next().await;
    // println!("Message: {:?}", message );
}