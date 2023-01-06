// 抢盘口做市策略, 最基础的做市策略，买一卖一抢单抢盘口, 赚买一卖一的差价.
// 比如现在卖1是60买1是70, 此策略会以65为中界，65以下布满买单，65以上布满卖单, 因为需要不停的调整订单布局，暂起名为高频逼近型
// 注意: 模拟测试GetTicker的买一卖一固定差价为1.6, 实际效果需要实盘测试

use bian_rs::{
    client::UFuturesWSClient,
    error::BianResult,
    response::{self, WebsocketResponse},
};

use crate::client::init_client;

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
        dbg!(stream.read_stream_single().unwrap());
    }
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
