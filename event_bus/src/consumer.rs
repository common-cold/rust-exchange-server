use std::time::Duration;

use common::{InsertTradeArgs, Order, UserBalance};
use db::{create_trade, update_order, update_user_balance};
use redis::{AsyncConnectionConfig, AsyncTypedCommands, aio::{ConnectionManager, MultiplexedConnection}, streams::StreamReadOptions};
use redis_service::RedisConnection;
use sqlx::{Pool, Postgres};
use tokio::time::interval;

use crate::{BALANCE_DLQ_STREAM_NAME, BALANCE_STREAM_NAME, ORDER_DLQ_STREAM_NAME, ORDER_STREAM_NAME, TRADE_DLQ_STREAM_NAME, TRADE_STREAM_NAME, producer::EventBusProducer};

pub struct EventBusConsumer {

}

impl EventBusConsumer {
    pub async fn run(pool: Pool<Postgres>, redis: RedisConnection) -> anyhow::Result<()> {
        let config = AsyncConnectionConfig::new()
            .set_response_timeout(None);

        let balance_connection = redis.client.get_multiplexed_async_connection_with_config(&config).await?;
        let order_connection = redis.client.get_multiplexed_async_connection_with_config(&config).await?;
        let trade_connection = redis.client.get_multiplexed_async_connection_with_config(&config).await?;

        let balance_db_pool = pool.clone();
        let order_db_pool = pool.clone();
        let trade_db_pool = pool.clone();

        let event_producer = EventBusProducer {
            redis_conn: redis.connection_manger
        };

        let balance_event_producer = event_producer.clone();
        let order_event_producer = event_producer.clone();
        let trade_event_producer = event_producer.clone();
        

        tokio::spawn(async move {
            let _ = EventBusConsumer::consume_balance_events(balance_connection, balance_db_pool, balance_event_producer).await;
        });

        tokio::spawn(async move {
            let _ = EventBusConsumer::consume_order_events(order_connection, order_db_pool, order_event_producer).await;
        });

        tokio::spawn(async move {
            let _ = EventBusConsumer::consume_trade_events(trade_connection, trade_db_pool, trade_event_producer).await;
        });

        Ok(())
    }

    pub async fn consume_balance_events(mut balance_connection: MultiplexedConnection, pool: Pool<Postgres>, event_producer: EventBusProducer) -> anyhow::Result<()> {
        let mut last_id = "0".to_string();
        loop {
            let response_option = match balance_connection.xread_options(
                &[BALANCE_STREAM_NAME], 
                &[&last_id],
                &StreamReadOptions::default().block(0)
            ).await {
                Ok(response) => response,
                Err(e) => {
                    println!("Balance consume task error: {}", e.to_string());
                    continue;
                }
            };

            if response_option.is_none() {
                continue;
            }

            let response = response_option.unwrap();
            for message in &response.keys[0].ids {
                let redis_value = message.map.get("value");
                if redis_value.is_none() {
                    continue;
                }
                let json_string: String = match redis::from_redis_value(redis_value.unwrap().clone()) {
                    Ok(response) => response,
                    Err(e) => {
                        println!("Balance consume task error: {}", e.to_string());
                        let _ = balance_connection.xdel(BALANCE_STREAM_NAME, &[message.id.clone()]).await;
                        continue;
                    }
                };
                let user_balance: UserBalance = match serde_json::from_str(&json_string) {
                    Ok(response) => response,
                    Err(e) => {
                        println!("Balance consume task error: {}", e.to_string());
                        let _ = balance_connection.xdel(BALANCE_STREAM_NAME, &[message.id.clone()]).await;
                        continue;
                    }
                };
                println!("{:#?}", user_balance);
                match update_user_balance(&pool, user_balance.clone()).await {
                    Err(e) => {
                        println!("Balance consume task error: {}", e.to_string());
                        match event_producer.publish_balance_dlq_event(user_balance.clone()).await {
                            Err(e) => {
                                println!("Balance consume task error: {}", e.to_string());
                            },
                            Ok(_) => {
                                let _ = balance_connection.xdel(BALANCE_STREAM_NAME, &[message.id.clone()]).await;
                            }
                        }
                        continue;
                    },
                    _ => {}
                };
                last_id = message.id.clone();
                let _ = balance_connection.xdel(BALANCE_STREAM_NAME, &[message.id.clone()]).await;
            }
        }
    }

    pub async fn consume_order_events(mut order_connection: MultiplexedConnection, pool: Pool<Postgres>, event_producer: EventBusProducer) -> anyhow::Result<()> {
        let mut last_id = "0".to_string();
        loop {
            let response_option = match order_connection.xread_options(
                &[ORDER_STREAM_NAME], 
                &[&last_id],
                &StreamReadOptions::default().block(0)
            ).await {
                Ok(response) => response,
                Err(e) => {
                    println!("Order consume task error: {}", e.to_string());
                    continue;
                }
            };

            if response_option.is_none() {
                continue;
            }

            let response = response_option.unwrap();
            

            for message in &response.keys[0].ids {
                let redis_value = message.map.get("value");
                if redis_value.is_none() {
                    continue;
                }
                let json_string: String = match redis::from_redis_value(redis_value.unwrap().clone()) {
                    Ok(response) => response,
                    Err(e) => {
                        println!("Order consume task error: {}", e.to_string());
                        let _ = order_connection.xdel(ORDER_STREAM_NAME, &[message.id.clone()]).await;
                        continue;
                    }
                };
                let order: Order = match serde_json::from_str(&json_string) {
                    Ok(response) => response,
                    Err(e) => {
                        println!("Order consume task error: {}", e.to_string());
                        let _ = order_connection.xdel(ORDER_STREAM_NAME, &[message.id.clone()]).await;
                        continue;
                    }
                };
                println!("{:#?}", order);
                match update_order(&pool, order.clone()).await {
                    Err(e) => {
                        println!("Order consume task error: {}", e.to_string());
                        match event_producer.publish_order_dlq_event(order.clone()).await {
                            Err(e) => {
                                println!("Order consume task error: {}", e.to_string());
                            },
                            Ok(_) => {
                                let _ = order_connection.xdel(ORDER_STREAM_NAME, &[message.id.clone()]).await;
                            }
                        }
                        continue;
                    },
                    _ => {},
                    
                };
                last_id = message.id.clone();
                let _ = order_connection.xdel(ORDER_STREAM_NAME, &[message.id.clone()]).await;
            }
        }
    }

    pub async fn consume_trade_events(mut trade_connection: MultiplexedConnection, pool: Pool<Postgres>, event_producer: EventBusProducer) -> anyhow::Result<()> {
        let mut last_id = "0".to_string();
        loop {
            let response_option = match trade_connection.xread_options(
                &[TRADE_STREAM_NAME], 
                &[&last_id],
                &StreamReadOptions::default().block(0)
            ).await {
                Ok(response) => response,
                Err(e) => {
                    println!("Trade consume task error: {}", e.to_string());
                    continue;
                }
            };

            if response_option.is_none() {
                continue;
            }

            let response = response_option.unwrap();

            for message in &response.keys[0].ids {
                let redis_value = message.map.get("value");
                if redis_value.is_none() {
                    continue;
                }
                let json_string: String = match redis::from_redis_value(redis_value.unwrap().clone()) {
                    Ok(response) => response,
                    Err(e) => {
                        println!("Trade consume task error: {}", e.to_string());
                        let _ = trade_connection.xdel(TRADE_STREAM_NAME, &[message.id.clone()]).await;
                        continue;
                    }
                };
                let trade_args: InsertTradeArgs = match serde_json::from_str(&json_string) {
                    Ok(response) => response,
                    Err(e) => {
                        println!("Trade consume task error: {}", e.to_string());
                        let _ = trade_connection.xdel(TRADE_STREAM_NAME, &[message.id.clone()]).await;
                        continue;
                    }
                };
                println!("{:#?}", trade_args);
                match create_trade(&pool, trade_args.clone()).await {
                    Err(e) => {
                        println!("Trade consume task error: {}", e.to_string());
                        match event_producer.publish_trade_dlq_event(trade_args.clone()).await {
                            Err(e) => {
                                println!("TRade consume task error: {}", e.to_string());
                            },
                            Ok(_) => {
                                let _ = trade_connection.xdel(TRADE_STREAM_NAME, &[message.id.clone()]).await;
                            }
                        }
                        continue;
                    },
                    _ => {}
                };
                last_id = message.id.clone();
                let _ = trade_connection.xdel(TRADE_STREAM_NAME, &[message.id.clone()]).await;
            }
        }
    }

    pub async fn run_dlq_consumers(redis: RedisConnection) {
        let balance_redis = redis.clone();
        let order_redis = redis.clone();
        let trade_redis = redis.clone();

        tokio::spawn(async move {
            let _ = EventBusConsumer::balance_dlq_consume_task(balance_redis).await;
        });

        tokio::spawn(async move {
            let _ = EventBusConsumer::order_dlq_consume_task(order_redis).await;
        });

        tokio::spawn(async move {
            let _ = EventBusConsumer::trade_dlq_consume_task(trade_redis).await;
        });
    }

    pub async fn balance_dlq_consume_task(redis: RedisConnection) {
        let mut interval = interval(Duration::from_secs(30));
        loop {
            interval.tick().await;
            match EventBusConsumer::consume_balance_dlq_events(redis.connection_manger.clone()).await {
                Err(e) => {
                    println!("Balance dlq consume task error: {}", e.to_string())
                },
                _ => {}
            };
        }
    }

    pub async fn order_dlq_consume_task(redis: RedisConnection) {
        let mut interval = interval(Duration::from_secs(30));
        loop {
            interval.tick().await;
            match EventBusConsumer::consume_order_dlq_events(redis.connection_manger.clone()).await {
                 Err(e) => {
                    println!("Order dlq consume task error: {}", e.to_string())
                },
                _ => {}
            };
        }
    }

    pub async fn trade_dlq_consume_task(redis: RedisConnection) {
        let mut interval = interval(Duration::from_secs(30));
        loop {
            interval.tick().await;
            match EventBusConsumer::consume_trade_dlq_events(redis.connection_manger.clone()).await {
                 Err(e) => {
                    println!("Trade dlq consume task error: {}", e.to_string())
                },
                _ => {}
            };
        }
    }


    pub async fn consume_balance_dlq_events(mut redis_conn: ConnectionManager) -> anyhow::Result<()> {
        let response_option = redis_conn.xread(
            &[BALANCE_DLQ_STREAM_NAME], 
            &["0"]
        ).await?;

        if response_option.is_none() {
            return Ok(())
        }

        let response = response_option.unwrap();
        let event_producer = EventBusProducer {
            redis_conn: redis_conn.clone()
        };

        for message in &response.keys[0].ids {
            let redis_value = message.map.get("value");
            if redis_value.is_none() {
                redis_conn.xdel(BALANCE_DLQ_STREAM_NAME, &[message.id.clone()]).await?;
                continue;
            }
            let json_string: String = match redis::from_redis_value(redis_value.unwrap().clone()) {
                Ok(response) => response,
                Err(e) => {
                    println!("Balance dlq consume task error: {}", e.to_string());
                    redis_conn.xdel(BALANCE_DLQ_STREAM_NAME, &[message.id.clone()]).await?;
                    continue;
                },
                
            };
            let user_balance: UserBalance = match serde_json::from_str(&json_string) {
                Ok(response) => response,
                Err(e) => {
                    println!("Balance dlq consume task error: {}", e.to_string());
                    redis_conn.xdel(BALANCE_DLQ_STREAM_NAME, &[message.id.clone()]).await?;
                    continue;
                },
            };
            
            event_producer.publish_balance_event(user_balance.clone()).await?;
            redis_conn.xdel(BALANCE_DLQ_STREAM_NAME, &[message.id.clone()]).await?;
        }
        Ok(())
    }

    pub async fn consume_order_dlq_events(mut redis_conn: ConnectionManager) -> anyhow::Result<()> {
        let response_option = redis_conn.xread(
            &[ORDER_DLQ_STREAM_NAME], 
            &["0"]
        ).await?;

        if response_option.is_none() {
            return Ok(())
        }

        let response = response_option.unwrap();
        let event_producer = EventBusProducer {
            redis_conn: redis_conn.clone()
        };

        for message in &response.keys[0].ids {
            let redis_value = message.map.get("value");
            if redis_value.is_none() {
                redis_conn.xdel(ORDER_DLQ_STREAM_NAME, &[message.id.clone()]).await?;
                continue;
            }
            let json_string: String = match redis::from_redis_value(redis_value.unwrap().clone()) {
                Ok(response) => response,
                Err(e) => {
                    println!("Order dlq consume task error: {}", e.to_string());
                    redis_conn.xdel(ORDER_DLQ_STREAM_NAME, &[message.id.clone()]).await?;
                    continue;
                },
            };
            let order: Order = match serde_json::from_str(&json_string) {
                Ok(response) => response,
                Err(e) => {
                    println!("Order dlq consume task error: {}", e.to_string());
                    redis_conn.xdel(ORDER_DLQ_STREAM_NAME, &[message.id.clone()]).await?;
                    continue;
                },
            };
            
            event_producer.publish_order_event(order.clone()).await?;
            let _ = redis_conn.xdel(ORDER_DLQ_STREAM_NAME, &[message.id.clone()]).await?;
        }
        Ok(())
    }

    pub async fn consume_trade_dlq_events(mut redis_conn: ConnectionManager) -> anyhow::Result<()> {
        let response_option = redis_conn.xread(
            &[TRADE_DLQ_STREAM_NAME], 
            &["0"]
        ).await?;

        if response_option.is_none() {
            return Ok(())
        }

        let response = response_option.unwrap();
        let event_producer = EventBusProducer {
            redis_conn: redis_conn.clone()
        };

        for message in &response.keys[0].ids {
            let redis_value = message.map.get("value");
            if redis_value.is_none() {
                redis_conn.xdel(ORDER_DLQ_STREAM_NAME, &[message.id.clone()]).await?;
                continue;
            }
            let json_string: String = match redis::from_redis_value(redis_value.unwrap().clone()) {
                Ok(response) => response,
                Err(e) => {
                    println!("Trade dlq consume task error: {}", e.to_string());
                    redis_conn.xdel(TRADE_DLQ_STREAM_NAME, &[message.id.clone()]).await?;
                    continue;
                },
            };
            let insert_trade_args: InsertTradeArgs = match serde_json::from_str(&json_string) {
                Ok(response) => response,
                Err(e) => {
                    println!("Trade dlq consume task error: {}", e.to_string());
                    redis_conn.xdel(TRADE_DLQ_STREAM_NAME, &[message.id.clone()]).await?;
                    continue;
                },
            };
            
            event_producer.publish_trade_event(insert_trade_args).await?;
            let _ = redis_conn.xdel(TRADE_DLQ_STREAM_NAME, &[message.id.clone()]).await?;
        }
        Ok(())
    }
}