use crate::{InsertTradeArgs, Order, UserBalance};


pub enum BalanceEvent {
    UpdateBalance(UserBalance),
    Shutdown
}

pub enum OrderEvent {
    UpdateOrder(Order),
    Shutdown
}

pub enum TradeEvent {
    InsertTrade(InsertTradeArgs),
    Shutdown
}