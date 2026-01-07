use common::Trade;
use async_trait::async_trait;

#[async_trait]
pub trait TradeTrait {}

impl TradeTrait for Trade {}
