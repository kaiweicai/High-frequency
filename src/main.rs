use bian_rs::{enums, client::{UFuturesWSClient, UFuturesHttpClient}, response::WebsocketResponse};
use std::{env, net::ToSocketAddrs};
use url;


#[tokio::main]
async fn main() {
    // let api_key = "xLZ0AtYGM5maQl1CKk9PzHV96eRE5TjZaoGbkdf4g4UGEeH6qv9AHWyEMT1BYBqd";
    // let secret_key = "iTV8dFhKYmJR9r7Ujj7WqRyzoYGX9PZJgRm2We6e0fJ8iyvPVCDO3sBy20uFrKXD";
    // // 默认 endpoint
    // let mut client = UFuturesHttpClient::default_endpoint(api_key.to_string(), secret_key.to_string());
    // client.base_url = url::Url::parse("https://fapi.binance.com").unwrap();
    // // 测试是否连通
    // let result = client.index_infos().await.unwrap();
    // dbg!("result is: {:?}", result);

    // let client = init_client();
    // let mut stream = client
    //     .kline("btcusdt".to_string(), enums::Interval::Min1)
    //     .unwrap();
    // for _ in 0..5 {
    //     dbg!(stream.read_stream_single().unwrap());
    // }
    // tokio::spawn(async move {
        high_frequency::grab_handicap::get_ws_order_book("btcusdt").await;
    // });
    println!("start main!");
    
}

// #[test]
// fn test_ws_kline() {
//     let client = init_client();
//     let mut stream = client
//         .kline("btcusdt".to_string(), enums::Interval::Min1)
//         .unwrap();
//     for _ in 0..5 {
//         dbg!(stream.read_stream_single().unwrap());
//     }
// }