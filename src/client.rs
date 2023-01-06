use std::{env, net::ToSocketAddrs};

use bian_rs::client::UFuturesWSClient;

pub fn init_client() -> UFuturesWSClient {
    dotenv::dotenv().unwrap();
    let proxy = env::var("WS_PROXY").expect("cant not find WS_PROXY env variable");
    let proxy = Some(proxy.to_socket_addrs().unwrap().next().unwrap());
    let profile = env::var("PROFILE").expect("cant not find PROFILE env variable");
    let base_url;
    if profile == crate::constants::DEV{
        base_url = crate::constants::TEST_BASE_WS_URL;
    }else{
        base_url = crate::constants::BASE_WS_URL;
    }
    let mut client = UFuturesWSClient::default_endpoint(proxy);
    client.base_url = url::Url::parse(base_url).unwrap();
    client
}