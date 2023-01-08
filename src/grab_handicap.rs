// 抢盘口做市策略, 最基础的做市策略，买一卖一抢单抢盘口, 赚买一卖一的差价.
// 比如现在卖1是60买1是70, 此策略会以65为中界，65以下布满买单，65以上布满卖单, 因为需要不停的调整订单布局，暂起名为高频逼近型
// 注意: 模拟测试GetTicker的买一卖一固定差价为1.6, 实际效果需要实盘测试

use std::{collections::HashMap, iter::Map};

use bian_rs::{
    client::UFuturesWSClient,
    error::BianResult,
    response::{self, WebsocketResponse, DepthOrder, WSFuturesDepth},
};
use serde::Deserialize;

use crate::client::init_client;

use lazy_static::lazy_static;

lazy_static! {
    
    static ref HASHMAP: HashMap<f64, DepthOrder> = {
        let mut m = HashMap::new();
        m
    };
    static ref COUNT: usize = HASHMAP.len();
}

// static mut order_book_map = HashMap::<f64,DepthOrder>::new();


// 订阅 wss://fstream.binance.com/stream?streams=btcusdt@depth
// 开始缓存收到的更新。同一个价位，后收到的更新覆盖前面的。
// 访问Rest接口 https://fapi.binance.com/fapi/v1/depth?symbol=BTCUSDT&limit=1000获得一个1000档的深度快照
// 将目前缓存到的信息中u< 步骤3中获取到的快照中的lastUpdateId的部分丢弃(丢弃更早的信息，已经过期)。
// 将深度快照中的内容更新到本地orderbook副本中，并从websocket接收到的第一个U <= lastUpdateId 且 u >= lastUpdateId 的event开始继续更新本地副本。
// 每一个新event的pu应该等于上一个event的u，否则可能出现了丢包，请从step3重新进行初始化。
// 每一个event中的挂单量代表这个价格目前的挂单量绝对值，而不是相对变化。
// 如果某个价格对应的挂单量为0，表示该价位的挂单已经撤单或者被吃，应该移除这个价位。

///获取order_book的websocket数据。
pub async fn get_ws_order_book(symbol: &str) {
    let ws_client = init_client();
    let ws_client = WsClient(ws_client);
    let mut stream = ws_client.orderbook_depth(symbol.to_string()).unwrap();
    //保存数据到内存中
    for _ in 0..5 {
        let otder_book = stream.read_stream_single().unwrap();
        // order_book_map.insert(otder_book.otder_book);
        dbg!(stream.read_stream_single().unwrap());
    }
}


pub async fn update_order_book(otder_book:WSFuturesDepth){

}



struct WsClient(UFuturesWSClient);

impl WsClient {
    /// 有限档深度信息
    ///
    /// 推送有限档深度信息。levels表示几档买卖单信息, 可选 5/10/20档
    /// Update Speed: 250ms 或 500ms 或 100ms
    fn orderbook_depth(
        &self,
        symbol: String,
    ) -> BianResult<impl WebsocketResponse<response::WSFuturesDepth>> {
        let channel = "depth".to_string();
        self.0.build_single(symbol, &channel)
    }
}

/// 有限档深度信息
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct WSFuturesOrderBook {
    /// 事件类型
    #[serde(rename = "e")]
    pub event_type: String,
    /// 事件推送时间
    #[serde(rename = "E")]
    pub event_time: i64,
    /// 交易时间
    #[serde(rename = "T")]
    pub trade_time: i64,
    /// 交易对
    #[serde(rename = "s")]
    pub symbol: String,
    /// 更新ID
    #[serde(rename = "u")]
    pub update_id: usize,
    /// ???
    #[serde(rename = "U")]
    pub upper_u: usize,
    /// ???
    pub pu: usize,
    /// 买方
    #[serde(rename = "b")]
    pub buy: HashMap<f64,DepthOrder>,
    /// 卖方
    #[serde(rename = "a")]
    pub sell: HashMap<f64,DepthOrder>,
}
