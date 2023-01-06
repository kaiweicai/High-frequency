use std::{env, net::ToSocketAddrs};

use bian_rs::client::UFuturesWSClient;

pub fn init_client() -> UFuturesWSClient {
    dotenv::dotenv().unwrap();
    let proxy = env::var("WS_PROXY").expect("cant not find WS_PROXY env variable");
    let proxy = Some(proxy.to_socket_addrs().unwrap().next().unwrap());
    let mut client = UFuturesWSClient::default_endpoint(proxy);
    client.base_url = url::Url::parse(crate::constants::TEST_BASE_WS_URL).unwrap();
    client
}