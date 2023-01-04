use bian_rs::{enums, client::{UFuturesWSClient, UFuturesHttpClient}, response::WebsocketResponse};
use std::{env, net::ToSocketAddrs};

#[tokio::main]
async fn main() {
    let api_key = "xLZ0AtYGM5maQl1CKk9PzHV96eRE5TjZaoGbkdf4g4UGEeH6qv9AHWyEMT1BYBqd";
    let secret_key = "iTV8dFhKYmJR9r7Ujj7WqRyzoYGX9PZJgRm2We6e0fJ8iyvPVCDO3sBy20uFrKXD";
    // 默认 endpoint
    let client = UFuturesHttpClient::default_endpoint(api_key.to_string(), secret_key.to_string());
    // 测试是否连通
    client.ping().await.unwrap();
}

fn init_client() -> UFuturesWSClient {
    // let base_url = "wss://stream.binance.com:9443";
    // dotenv::dotenv().unwrap();
    // let proxy = env::var("WS_PROXY").expect("cant not find WS_PROXY env variable");
    let proxy = "127.0.0.1:7890";
    let proxy = Some(proxy.to_socket_addrs().unwrap().next().unwrap());
    UFuturesWSClient::default_endpoint(proxy)
}

#[test]
fn test_ws_kline() {
    let client = init_client();
    let mut stream = client
        .kline("btcusdt".to_string(), enums::Interval::Min1)
        .unwrap();
    for _ in 0..5 {
        dbg!(stream.read_stream_single().unwrap());
    }
}