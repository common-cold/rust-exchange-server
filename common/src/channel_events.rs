use std::collections::HashMap;

use tokio::sync::mpsc::Sender;
use uuid::Uuid;

use crate::{CreateOrderArgs, InsertTradeArgs, OnRampArgs, Order, Orderbook, UserBalance};


#[derive(Debug)]
pub enum EngineIx {
    CreateLimitOrder(CreateOrderArgs),
    CreateMarketOrder(CreateOrderArgs),
    CancelOrder {
        order_id: Uuid
    },
    OnRamp(OnRampArgs),
    State(Sender<AcknowledgementEvent>),
    Shutdown(Sender<AcknowledgementEvent>),
    Flush(Sender<AcknowledgementEvent>),
    Debug
}


pub enum BalanceEvent {
    UpdateBalance(UserBalance),
    Shutdown(Sender<AcknowledgementEvent>),
    Flush(Sender<AcknowledgementEvent>)
}

pub enum OrderEvent {
    UpdateOrder(Order),
    Shutdown(Sender<AcknowledgementEvent>),
    Flush(Sender<AcknowledgementEvent>)
}

pub enum TradeEvent {
    InsertTrade(InsertTradeArgs),
    Shutdown(Sender<AcknowledgementEvent>), 
    Flush(Sender<AcknowledgementEvent>)
}

pub enum AcknowledgementEvent {
    Shutdown,
    Flush,
    State((Orderbook, HashMap<Uuid, UserBalance>))
}