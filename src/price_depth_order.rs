use bian_rs::response::DepthOrder;

/// (价格, 数量,lastUpdateId)
#[derive(Debug)]
pub struct PriceDepthOrder(pub f64, pub f64,pub usize);

impl PriceDepthOrder {
    pub fn new(depth_order:&DepthOrder,last_update_id:usize)->Self{
        PriceDepthOrder(depth_order.0,depth_order.1,last_update_id)
    }
}