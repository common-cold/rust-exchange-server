use std::collections::HashMap;

use anyhow::Ok;
use bigdecimal::BigDecimal;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::service::{CreateOrderArgs, Order, Side};


#[derive(Serialize, Deserialize, Clone)]
pub struct UserBalance {
    pub id: Uuid,
    pub user_id: Uuid,
    pub free_base_qty: BigDecimal,
    pub free_quote_qty: BigDecimal,
    pub locked_base_qty: BigDecimal,
    pub locked_quote_qty: BigDecimal,
}

impl UserBalance {
    pub fn init_user_balances(balances: Vec<UserBalance>) -> anyhow::Result<HashMap<Uuid, UserBalance>>{
        let mut balance_map: HashMap<Uuid, UserBalance> = HashMap::new();
        
        for balance in balances.iter() {
            balance_map.insert(balance.id, balance.clone());
        }

        Ok(balance_map)
    }

    pub fn credit_locked_quote_qty(&mut self, amount: &BigDecimal) -> anyhow::Result<()> {
        self.locked_quote_qty += amount;
        Ok(())
    }

    pub fn credit_locked_base_qty(&mut self, amount: &BigDecimal) -> anyhow::Result<()> {
        self.locked_base_qty += amount;
        Ok(())
    }

    pub fn lock_free_quote_qty(&mut self, amount: &BigDecimal) -> anyhow::Result<()> {
        if self.free_quote_qty < *amount {
            return Err(anyhow::anyhow!("Insufficient free quote qty to lock"));
        }

        self.locked_quote_qty += amount;
        self.free_quote_qty -= amount;
        Ok(())
    }

    pub fn lock_free_base_qty(&mut self, amount: &BigDecimal) -> anyhow::Result<()> {
        if self.free_base_qty < *amount {
            return Err(anyhow::anyhow!("Insufficient free base qty to lock"));
        }

        self.locked_base_qty += amount;
        self.free_base_qty -= amount;
        Ok(())
    }

    pub fn lock_funds(&mut self, args: &CreateOrderArgs) -> anyhow::Result<BigDecimal> {
        match args.side {
            Side::Bid => {
                let free_quote_qty = self.free_quote_qty.clone();
                let quote_qty_to_lock = free_quote_qty.clone().min(args.quote_qty.clone());   
                let deposit_amount = &args.quote_qty - quote_qty_to_lock;
                self.credit_locked_quote_qty(&deposit_amount)?;
                self.lock_free_quote_qty(&free_quote_qty)?;
                Ok(deposit_amount)
            }
            Side::Ask => {
                let free_base_qty = self.free_base_qty.clone();
                let base_qty_to_lock = free_base_qty.clone().min(args.base_qty.clone());   
                let deposit_amount = &args.base_qty - base_qty_to_lock;
                self.credit_locked_base_qty(&deposit_amount)?;
                self.lock_free_base_qty(&free_base_qty)?;  
                Ok(deposit_amount)                    
            }
        }
    }

    pub fn update_balance(&mut self, side: Side, price: &BigDecimal, trade_qty: &BigDecimal) -> anyhow::Result<()> {
        match side {
            Side::Bid => {
                self.free_base_qty += trade_qty;
                self.locked_quote_qty -= &(trade_qty * price);
            }
            Side::Ask => {
                self.free_quote_qty += &(trade_qty * price);
                self.locked_base_qty -= trade_qty;
            }
        };
        Ok(())
    }

}