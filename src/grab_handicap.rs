// 抢盘口做市策略, 最基础的做市策略，买一卖一抢单抢盘口, 赚买一卖一的差价.
// 比如现在卖1是60买1是70, 此策略会以65为中界，65以下布满买单，65以上布满卖单, 因为需要不停的调整订单布局，暂起名为高频逼近型
// 注意: 模拟测试GetTicker的买一卖一固定差价为1.6, 实际效果需要实盘测试
use std::{
    collections::{BTreeMap, HashMap},
    fmt,
    iter::Map,
    ops::{Mul, Index},
    sync::{Arc, Mutex},
};

use bian_rs::{
    client::UFuturesWSClient,
    error::BianResult,
    params,
    response::{self, DepthOrder, FuturesDepth, WSFuturesDepth, WebsocketResponse},
};
use serde::{
    de::{SeqAccess, Unexpected, Visitor},
    Deserialize, Deserializer,
};

use crate::{
    client::{self, init_ws_client},
    price_depth_order::PriceDepthOrder,
};

use lazy_static::lazy_static;

type OrderBookTreeMap = Arc<Mutex<BTreeMap<u64, PriceDepthOrder>>>;
type OrderBookDepth = Arc<Mutex<Vec<DepthOrder>>>;
static BASE_DECIMAL: f64 = 1000000f64;
lazy_static! {
    static ref order_book_bid_map: OrderBookTreeMap = { Arc::new(Mutex::new(BTreeMap::new())) };
    static ref order_book_ask_map: OrderBookTreeMap = { Arc::new(Mutex::new(BTreeMap::new())) };
    static ref ask_book_depth: OrderBookDepth = { Arc::new(Mutex::new(Vec::new())) };
    static ref bid_book_depth: OrderBookDepth = { Arc::new(Mutex::new(Vec::new())) };
}

// static mut order_book_map = HashMap::<f64,DepthOrder>::new();

impl bid_book_depth {
    pub fn update_order(&mut self, updateOrder: Vec<DepthOrder>) {}
}

// 如何正确在本地维护一个orderbook副本 信息来源于:https://binance-docs.github.io/apidocs/futures/cn/#1654ad2dd2
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
    init_order_book(symbol).await;

    let ws_client = client::init_ws_client();
    let ws_client = WsClient(ws_client);
    let mut stream = ws_client.orderbook_depth(symbol.to_string()).unwrap();
    //保存数据到内存中
    loop {
        match stream.read_stream_single() {
            Ok(order_book_update) => {
                // order_book_map.insert(order_book.order_book);
                println!("order_book is:{:#?}", order_book_update.update_id);
                order_book_update
                    .buy
                    .iter()
                    .filter(|order| order.1 > 0f64)// 去掉没有量的单
                    .for_each(|order| {
                        let price_depth_order =
                            PriceDepthOrder::new(order, order_book_update.update_id);
                        order_book_bid_map.clone().lock().unwrap().insert(
                            price_depth_order.0.mul(BASE_DECIMAL) as u64,
                            price_depth_order,
                        );
                    });
                order_book_update
                    .sell
                    .iter()
                    .filter(|order| order.1 > 0f64)
                    .for_each(|order| {
                        let price_depth_order =
                            PriceDepthOrder::new(order, order_book_update.update_id);
                        order_book_ask_map.clone().lock().unwrap().insert(
                            price_depth_order.0.mul(BASE_DECIMAL) as u64,
                            price_depth_order,
                        );
                    });
            }
            Err(e) => {
                println!("get order book error:{:?}", e);
            }
        }
        println!(           "order_book_ask_map is:{:?}",           order_book_ask_map.clone().lock().unwrap().last_key_value().unwrap().1       );
        println!(            "order_book_bid_map is:{:?}",            order_book_bid_map.clone().lock().unwrap().last_key_value().unwrap().1      );
        // println!(           "order_book_ask_map is:{:?}",           order_book_ask_map.clone().lock().unwrap()        );
        // println!(            "order_book_bid_map is:{:?}",            order_book_bid_map.clone().lock().unwrap()        );
    }
}

pub async fn init_order_book(symbol: &str) {
    let mut init_orderbook = get_init_order_book(symbol).await;
    let last_update_id = init_orderbook.last_update_id;
    // 将目前缓存到的信息中u< 步骤3中获取到的快照中的lastUpdateId的部分丢弃(丢弃更早的信息，已经过期)。
    order_book_bid_map.clone().lock().unwrap().retain(|_,v|v.2>last_update_id);
    order_book_ask_map.clone().lock().unwrap().retain(|_,v|v.2>last_update_id);
    println!("ask_book_depth is:{:?}",init_orderbook.last_update_id);
    // 将深度快照中的内容更新到本地orderbook副本中，并从websocket接收到的第一个U <= lastUpdateId 且 u >= lastUpdateId 的event开始继续更新本地副本。
    init_orderbook.asks.iter().for_each(|order|{
        let price_depth_order = PriceDepthOrder::new(order, last_update_id);
        order_book_ask_map.clone().lock().unwrap().insert(
            price_depth_order.0.mul(BASE_DECIMAL) as u64,
            price_depth_order,
        );
    });
    init_orderbook.bids.iter().for_each(|order|{
        let price_depth_order = PriceDepthOrder::new(order, last_update_id);
        order_book_bid_map.clone().lock().unwrap().insert(
            price_depth_order.0.mul(BASE_DECIMAL) as u64,
            price_depth_order,
        );
    });
    // println!(           "order_book_ask_map is:{:?}",           order_book_ask_map.clone().lock().unwrap().last_key_value().unwrap().1       );
    // println!(            "order_book_bid_map is:{:?}",            order_book_bid_map.clone().lock().unwrap().last_key_value().unwrap().1      );
    println!(           "order_book_ask_map is:{:?}",           order_book_ask_map.clone().lock().unwrap()        );
        println!(            "order_book_bid_map is:{:?}",            order_book_bid_map.clone().lock().unwrap()        );
}

pub async fn get_init_order_book(symbol: &str) -> FuturesDepth {
    let client = client::init_client();
    let param = params::PDepth {
        symbol: symbol.to_string(),
        limit: 500,
    };
    let order_book = client.depth(param).await.unwrap();
    order_book
    // dbg!(client.depth(param).await.unwrap());
}

#[tokio::test]
async fn test_depth() {
    let client = client::init_client();
    let param = params::PDepth {
        symbol: "BTCUSDT".to_string(),
        limit: 500,
    };
    dbg!(client.depth(param).await.unwrap());
}

pub async fn update_order_book(order_book: WSFuturesDepth) {}

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

// /// 有限档深度信息
// #[derive(Debug, Deserialize)]
// #[serde(rename_all = "camelCase")]
// pub struct WSFuturesOrderBook {
//     /// 事件类型
//     #[serde(rename = "e")]
//     pub event_type: String,
//     /// 事件推送时间
//     #[serde(rename = "E")]
//     pub event_time: i64,
//     /// 交易时间
//     #[serde(rename = "T")]
//     pub trade_time: i64,
//     /// 交易对
//     #[serde(rename = "s")]
//     pub symbol: String,
//     /// 更新ID
//     #[serde(rename = "u")]
//     pub update_id: usize,
//     /// ???
//     #[serde(rename = "U")]
//     pub upper_u: usize,
//     /// ???
//     pub pu: usize,
//     /// 买方
//     #[serde(rename = "b")]
//     pub buy: BTreeMap<PriceOrder,DepthOrder>,
//     /// 卖方
//     #[serde(rename = "a")]
//     pub sell: BTreeMap<PriceOrder,DepthOrder>,
// }

// #[derive(Debug)]
// pub struct PriceOrder(f64);

// impl Ord for PriceOrder{
//     fn cmp(&self, other: &Self) -> std::cmp::Ordering {
//         return self.0.cmp(other.0)
//     }

//     fn max(self, other: Self) -> Self
//     where
//         Self: Sized,
//         Self: ~const std::marker::Destruct,
//     {
//         // HACK(fee1-dead): go back to using `self.max_by(other, Ord::cmp)`
//         // when trait methods are allowed to be used when a const closure is
//         // expected.
//         match self.cmp(&other) {
//             std::cmp::Ordering::Less | std::cmp::Ordering::Equal => other,
//             std::cmp::Ordering::Greater => self,
//         }
//     }

//     fn min(self, other: Self) -> Self
//     where
//         Self: Sized,
//         Self: ~const std::marker::Destruct,
//     {
//         // HACK(fee1-dead): go back to using `self.min_by(other, Ord::cmp)`
//         // when trait methods are allowed to be used when a const closure is
//         // expected.
//         match self.cmp(&other) {
//             std::cmp::Ordering::Less | std::cmp::Ordering::Equal => self,
//             std::cmp::Ordering::Greater => other,
//         }
//     }

//     fn clamp(self, min: Self, max: Self) -> Self
//     where
//         Self: Sized,
//         Self: ~const std::marker::Destruct,
//         Self: ~const PartialOrd,
//     {
//         assert!(min <= max);
//         if self < std::cmp::min {
//             std::cmp::min
//         } else if self > std::cmp::max {
//             std::cmp::max
//         } else {
//             self
//         }
//     }
// }

// impl<'de> Deserialize<'de> for PriceOrder {
//     fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
//     where
//         D: Deserializer<'de>,
//     {
//         deserializer.deserialize_tuple(1, PriceOrderVisitor)
//     }
// }

// struct PriceOrderVisitor;

// impl<'de> Visitor<'de> for PriceOrderVisitor {
//     type Value = PriceOrder;
//     fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
//         formatter.write_str("a tuple of (String)")
//     }

//     fn visit_seq<A>(self, mut seq: A) -> Result<Self::Value, A::Error>
//     where
//         A: SeqAccess<'de>,
//     {
//         let first: &'de str = seq
//             .next_element()?
//             .ok_or_else(|| serde::de::Error::invalid_value(Unexpected::Option, &"first element"))?;
//         let first_val = first.parse::<f64>().map_err(|_| {
//             serde::de::Error::invalid_value(Unexpected::Str(first), &"first element")
//         })?;
//         Ok(PriceOrder(first_val))
//     }
// }
