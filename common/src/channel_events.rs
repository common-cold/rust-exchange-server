use std::collections::HashMap;

use tokio::sync::mpsc::Sender;
use uuid::Uuid;

use crate::{CreateOrderArgs, InsertTradeArgs, Order, Orderbook, UserBalance};


#[derive(Debug)]
pub enum EngineIx {
    CreateLimitOrder(CreateOrderArgs),
    CreateMarketOrder(CreateOrderArgs),
    CancelOrder {
        key: String
    },
    State(Sender<AcknowledgementEvent>),
    Shutdown(Sender<AcknowledgementEvent>),
    Flush(Sender<AcknowledgementEvent>)
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