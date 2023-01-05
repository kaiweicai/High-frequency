// 抢盘口做市策略, 最基础的做市策略，买一卖一抢单抢盘口, 赚买一卖一的差价.
// 比如现在卖1是60买1是70, 此策略会以65为中界，65以下布满买单，65以上布满卖单, 因为需要不停的调整订单布局，暂起名为高频逼近型
// 注意: 模拟测试GetTicker的买一卖一固定差价为1.6, 实际效果需要实盘测试

use std::{env, net::ToSocketAddrs};

use bian_rs::client::UFuturesWSClient;

use crate::constants::TEST_BASE_WS_URL;

pub fn init_client() -> UFuturesWSClient {
    dotenv::dotenv().unwrap();
    let proxy = env::var("WS_PROXY").expect("cant not find WS_PROXY env variable");
    let proxy = Some(proxy.to_socket_addrs().unwrap().next().unwrap());
    let mut client = UFuturesWSClient::default_endpoint(proxy);
    client.base_url = url::Url::parse(TEST_BASE_WS_URL).unwrap();
    client
}