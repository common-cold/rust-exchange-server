use std::collections::HashMap;

use bigdecimal::BigDecimal;
use serde::{Deserialize, Serialize};
use uuid::Uuid;


#[derive(Serialize, Deserialize, Clone)]
pub struct UserBalance {
    pub id: Uuid,
    pub user_id: Uuid,
    pub free_base_qty: BigDecimal,
    pub free_quote_qty: BigDecimal,
    pub locked_base_qty: BigDecimal,
    pub locked_quote_qty: BigDecimal
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

}