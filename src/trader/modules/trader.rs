use std::sync::{Arc, Mutex};

use polars::prelude::*;
use tokio::{spawn, task::JoinHandle, time::sleep};

use crate::{
    shared::csv::save_csv,
    trader::{
        enums::{
            balance::Balance,
            log_level::LogLevel,
            modifiers::{leverage::Leverage, price_level::PriceLevel},
            order_action::OrderAction,
            side::Side,
            signal_category::SignalCategory,
            trade_status::TradeStatus,
            trading_data_update::TradingDataUpdate,
        },
        errors::{CustomError, Error},
        functions::{
            current_datetime, current_timestamp_ms, get_symbol_close_col, timestamp_minute_end,
            timestamp_minute_start, update_position_data_on_faulty_exchange_ws,
        },
        models::{
            behavior_subject::BehaviorSubject, execution::Execution, order::Order, trade::Trade,
            trading_settings::TradingSettings,
        },
        traits::exchange::Exchange,
    },
};

use super::{data_feed::DataFeed, performance::Performance, strategy::Strategy};
use futures_util::StreamExt;

#[derive(Clone)]
pub struct Trader {
    pub data_feed: DataFeed,
    pub strategy_arc: Arc<Mutex<Strategy>>,
    pub performance_arc: Arc<Mutex<Performance>>,
    pub temp_executions_arc: Arc<Mutex<Vec<Execution>>>,
    pub trading_settings_arc: Arc<Mutex<TradingSettings>>,
    pub exchange_socket_error_arc: Arc<Mutex<Option<i64>>>,
    pub exchange_listener: BehaviorSubject<Box<dyn Exchange + Send + Sync>>,
    pub current_balance_listener: BehaviorSubject<Balance>,
    pub update_balance_listener: BehaviorSubject<Option<Balance>>,
    pub update_order_listener: BehaviorSubject<Option<OrderAction>>,
    pub update_executions_listener: BehaviorSubject<Vec<Execution>>,
    pub signal_listener: BehaviorSubject<Option<SignalCategory>>,
    pub current_trade_listener: BehaviorSubject<Option<Trade>>,
    pub trading_data_listener: BehaviorSubject<DataFrame>,
    pub trading_data_update_listener: BehaviorSubject<TradingDataUpdate>,
    pub leverage_listener: BehaviorSubject<Leverage>,
    pub log_level: LogLevel,
    pub is_data_gather_only: bool,
}

impl Trader {
    pub fn new(
        data_feed: DataFeed,
        strategy_arc: &Arc<Mutex<Strategy>>,
        performance_arc: &Arc<Mutex<Performance>>,
        trading_settings_arc: &Arc<Mutex<TradingSettings>>,
        exchange_socket_error_arc: &Arc<Mutex<Option<i64>>>,
        exchange_listener: &BehaviorSubject<Box<dyn Exchange + Send + Sync>>,
        current_balance_listener: &BehaviorSubject<Balance>,
        update_balance_listener: &BehaviorSubject<Option<Balance>>,
        update_order_listener: &BehaviorSubject<Option<OrderAction>>,
        update_executions_listener: &BehaviorSubject<Vec<Execution>>,
        signal_listener: &BehaviorSubject<Option<SignalCategory>>,
        trading_data_listener: &BehaviorSubject<DataFrame>,
        trading_data_update_listener: &BehaviorSubject<TradingDataUpdate>,
        current_trade_listener: &BehaviorSubject<Option<Trade>>,
        leverage_listener: &BehaviorSubject<Leverage>,
        log_level: &LogLevel,
        is_data_gather_only: bool,
    ) -> Trader {
        Trader {
            data_feed,
            strategy_arc: strategy_arc.clone(),
            performance_arc: performance_arc.clone(),
            temp_executions_arc: Arc::new(Mutex::new(Vec::new())),
            trading_settings_arc: trading_settings_arc.clone(),
            exchange_socket_error_arc: exchange_socket_error_arc.clone(),
            exchange_listener: exchange_listener.clone(),
            current_balance_listener: current_balance_listener.clone(),
            signal_listener: signal_listener.clone(),
            update_balance_listener: update_balance_listener.clone(),
            update_order_listener: update_order_listener.clone(),
            update_executions_listener: update_executions_listener.clone(),
            trading_data_update_listener: trading_data_update_listener.clone(),
            trading_data_listener: trading_data_listener.clone(),
            current_trade_listener: current_trade_listener.clone(),
            leverage_listener: leverage_listener.clone(),
            log_level: log_level.clone(),
            is_data_gather_only,
        }
    }

    pub async fn init(self) {
        let exchange_listener = self.exchange_listener.clone();
        let trading_settings_arc = self.trading_settings_arc.clone();
        let leverage_listener = self.leverage_listener.clone();

        // TODO: CHECK THIS QUERY
        // let leverage_change_handle = spawn(async move {
        //     let mut subscription = leverage_listener.subscribe();
        //     while let Some(leverage) = subscription.next().await {
        //         let exchange_binding = exchange_listener.value();
        //         let result = exchange_binding.set_leverage(leverage.clone()).await;
        //         match result {
        //             Ok(success) => {
        //                 if success {
        //                     let mut settings_guard = trading_settings_arc
        //                         .lock()
        //                         .expect("leverage_change_handle -> trading setting deadlock");
        //                     settings_guard.leverage = leverage;
        //                 }
        //             }
        //             Err(error) => {
        //                 println!("leverage_change_handle error {:?}", error);
        //             }
        //         }
        //     }
        // });

        let signal_listener = self.signal_listener.clone();
        let current_trade_listener = self.current_trade_listener.clone();
        let exchange_listener = self.exchange_listener.clone();
        let trading_data_listener = self.trading_data_listener.clone();
        let current_balance_listener = self.current_balance_listener.clone();
        let trading_settings_arc = self.trading_settings_arc.clone();

        let signal_handle = get_signal_handle(
            signal_listener,
            current_trade_listener,
            exchange_listener,
            trading_data_listener,
            current_balance_listener,
            trading_settings_arc,
            self.is_data_gather_only,
        )
        .await;

        let update_balance_listener = self.update_balance_listener.clone();
        let current_balance_listener: BehaviorSubject<Balance> =
            self.current_balance_listener.clone();

        let update_balance_handle =
            get_update_balance_handle(update_balance_listener, current_balance_listener).await;

        let update_order_listener = self.update_order_listener.clone();
        let temp_executions_arc = self.temp_executions_arc.clone();
        let current_trade_listener = self.current_trade_listener.clone();
        let trading_settings_arc = self.trading_settings_arc.clone();

        let update_order_handle = get_update_order_handle(
            update_order_listener,
            current_trade_listener,
            temp_executions_arc,
            trading_settings_arc,
        )
        .await;

        let update_executions_listener = self.update_executions_listener.clone();
        let temp_executions_arc = self.temp_executions_arc.clone();

        let update_executions_handle =
            get_update_executions_handle(update_executions_listener, temp_executions_arc).await;

        let strategy_arc = self.strategy_arc.clone();
        let performance_arc = self.performance_arc.clone();
        let exchange_socket_error_arc = self.exchange_socket_error_arc.clone();
        let temp_executions_arc = self.temp_executions_arc.clone();
        let trading_data_listener = self.trading_data_listener.clone();
        let trading_data_update_listener = self.trading_data_update_listener.clone();
        let exchange_listener = self.exchange_listener.clone();
        let current_trade_listener = self.current_trade_listener.clone();
        let current_balance_listener = self.current_balance_listener.clone();
        let signal_listener = self.signal_listener.clone();
        let update_balance_listener = self.update_balance_listener.clone();
        let update_order_listener = self.update_order_listener.clone();
        let update_executions_listener = self.update_executions_listener.clone();

        let trading_data_handle = get_process_trading_data_handle(
            strategy_arc,
            performance_arc,
            exchange_socket_error_arc,
            temp_executions_arc,
            trading_data_listener,
            trading_data_update_listener,
            exchange_listener,
            current_trade_listener,
            update_balance_listener,
            update_order_listener,
            update_executions_listener,
            current_balance_listener,
            signal_listener,
        );

        let mut data_feed = self.data_feed.clone();
        let data_feed_handle = tokio::spawn(async move {
            let _ = data_feed.init().await;
        });

        let current_trade_listener = self.current_trade_listener.clone();
        let trading_data_listener = self.trading_data_listener.clone();
        let current_balance_listener = self.current_balance_listener.clone();
        let signal_listener = self.signal_listener.clone();

        let current_trade_update_handle = get_current_trade_update_handle(
            current_trade_listener,
            trading_data_listener,
            current_balance_listener,
            signal_listener,
        )
        .await;

        let _ = current_trade_update_handle.await;
        let _ = trading_data_handle.await;
        let _ = update_balance_handle.await;
        let _ = update_order_handle.await;
        let _ = update_executions_handle.await;
        let _ = data_feed_handle.await;
        let _ = signal_handle.await;
    }
}

async fn get_current_trade_update_handle(
    current_trade_listener: BehaviorSubject<Option<Trade>>,
    trading_data_listener: BehaviorSubject<DataFrame>,
    current_balance_listener: BehaviorSubject<Balance>,
    signal_listener: BehaviorSubject<Option<SignalCategory>>,
) -> JoinHandle<()> {
    spawn(async move {
        let mut subscription = current_trade_listener.subscribe();
        while let Some(current_trade) = subscription.next().await {
            if current_trade.is_none() {
                continue;
            }

            let current_trade = current_trade.unwrap();

            let trade_status = current_trade.status();

            if trade_status == TradeStatus::Cancelled || trade_status == TradeStatus::Closed {
                if trade_status == TradeStatus::Closed {
                    let close_order = current_trade.clone().close_order.unwrap();
                    let (pnl, returns) = current_trade.calculate_pnl_and_returns();
                    println!(
                        "\n{:?} | 📕 Closed Order {:?} side ({:?} units), profit/loss: {}, returns: {}",
                        current_datetime(),
                        current_trade.open_order.side,
                        &close_order.units,
                        pnl,
                        returns
                    );
                } else {
                    println!(
                        "\n{:?} | ❌ Current Order side {:?} cancelled successfully!",
                        current_datetime(),
                        current_trade.open_order.side,
                    );
                }

                let trading_data = trading_data_listener.value();
                let updated_trading_data = on_close_update_trading_data(
                    trading_data,
                    &current_balance_listener,
                    &signal_listener,
                    &current_trade_listener,
                )
                .expect(
                    "get_current_trade_update_handle -> on_close_update_trading_data unwrap failed",
                );
                if updated_trading_data.is_some() {
                    let updated_trading_data = updated_trading_data.unwrap();
                    trading_data_listener.next(updated_trading_data);
                }

                current_trade_listener.next(None);
            }
        }
    })
}

async fn get_signal_handle(
    signal_listener: BehaviorSubject<Option<SignalCategory>>,
    current_trade_listener: BehaviorSubject<Option<Trade>>,
    exchange: BehaviorSubject<Box<dyn Exchange + Send + Sync>>,
    trading_data_listener: BehaviorSubject<DataFrame>,
    current_balance: BehaviorSubject<Balance>,
    trading_settings_arc: Arc<Mutex<TradingSettings>>,
    is_data_gather_only: bool,
) -> JoinHandle<()> {
    spawn(async move {
        let mut subscription = signal_listener.subscribe();
        while let Some(signal_opt) = subscription.next().await {
            if signal_opt.is_none() || is_data_gather_only {
                continue;
            }
            let signal = signal_opt.expect("get_signal_handle -> unwraping signal");
            if signal == SignalCategory::KeepPosition {
                continue;
            }
            match process_last_signal(
                signal,
                &current_trade_listener,
                &exchange,
                &trading_data_listener,
                &current_balance,
                &trading_settings_arc,
            )
            .await
            {
                Ok(()) => {}
                Err(error) => {
                    println!("process_last_signal error {:?}", error);
                }
            }
        }
    })
}

async fn process_last_signal(
    signal: SignalCategory,
    current_trade_listener: &BehaviorSubject<Option<Trade>>,
    exchange: &BehaviorSubject<Box<dyn Exchange + Send + Sync>>,
    trading_data: &BehaviorSubject<DataFrame>,
    current_balance: &BehaviorSubject<Balance>,
    trading_settings_arc: &Arc<Mutex<TradingSettings>>,
) -> Result<(), Error> {
    let current_trade = current_trade_listener.value();

    let exchange = exchange.value();
    let traded_contract = exchange.get_traded_contract();
    let close_col = get_symbol_close_col(&traded_contract.symbol);
    let trading_data_binding = trading_data.value();
    let last_price = trading_data_binding
        .column(&close_col)?
        .f64()?
        .into_no_null_iter()
        .last()
        .expect("process_last_signal -> SignalCategory::GoLong -> missing last price");

    let trading_settings;
    {
        let trading_settings_guard = trading_settings_arc.lock().expect(
            "process_last_signal -> SignalCategory::GoLong -> trading_settings_guard deadlock",
        );
        trading_settings = trading_settings_guard.clone();
    }

    if let Some(mut current_trade) = current_trade {
        let current_trade_status = current_trade.status();
        match current_trade_status {
            TradeStatus::New => {
                if (signal == SignalCategory::CloseLong && current_trade.open_order.side == Side::Buy)
                    || (signal == SignalCategory::CloseShort
                        && current_trade.open_order.side == Side::Sell)
                    || (signal == SignalCategory::ClosePosition
                        && current_trade.open_order.side != Side::Nil)
                {
                    match exchange
                        .cancel_order(current_trade.open_order.id.clone())
                        .await
                    {
                        Ok(cancel_result) => {
                            if cancel_result {
                                println!(
                                    "\n{:?} | ⚠️ Current order {:?} position, without executions, will be cancelled as it received a close signal.",
                                    current_datetime(), 
                                    current_trade.open_order.side
                                );
                                Ok(())
                            } else {
                                let error =
                                    CustomError::new("TradeStatus::New -> Cancel Idle Position -> cancel order returned false".to_string());
                                Err(Error::CustomError(error))
                            }
                        }
                        Err(error) => {
                            let error = format!("TradeStatus::New -> Cancel Idle Position -> cancel result failed! {:?}", error);
                            let error = Error::CustomError(CustomError::new(error));
                            Err(error)
                        }
                    }
                } else if (signal == SignalCategory::GoLong
                    && current_trade.open_order.side == Side::Sell)
                    || (signal == SignalCategory::GoShort && current_trade.open_order.side == Side::Buy)
                {
                    match exchange
                        .cancel_order(current_trade.open_order.id.clone())
                        .await
                    {
                        Ok(cancel_result) => {
                            if cancel_result {
                                println!(
                                    "\n{:?} | ⚠️ Current idle order {:?} position, without executions, will be cancelled as it received an opposite side open signal.",
                                    current_datetime(),
                                    current_trade.open_order.side
                                );

                                let wallet_balance = current_balance.value().wallet_balance;

                                match open_order(
                                    trading_settings,
                                    exchange,
                                    if signal == SignalCategory::GoLong {
                                        Side::Buy
                                    } else {
                                        Side::Sell
                                    },
                                    wallet_balance,
                                    last_price,
                                )
                                .await
                                {
                                    Ok(()) => {
                                        println!(
                                            "\n{:?} | ♻️ Current idle order, {:?} position, will be recycled as it received an opposite side open signal.",
                                            current_datetime(),
                                            current_trade.open_order.side
                                        );
                                        Ok(())
                                    }
                                    Err(error) => {
                                        let error = format!("TradeStatus::New -> Recycle Idle Position -> open new position failed! {:?}", error);
                                        let error = Error::CustomError(CustomError::new(error));
                                        Err(error)
                                    }
                                }
                            } else {
                                let error =
                                    CustomError::new("TradeStatus::New -> Recycle Idle Position -> cancel order returned false".to_string());
                                Err(Error::CustomError(error))
                            }
                        }
                        Err(error) => {
                            let error = format!("TradeStatus::New -> Revert Idle Position -> cancel result failed! {:?}", error);
                            let error = Error::CustomError(CustomError::new(error));
                            Err(error)
                        }
                    }
                } else {
                    Ok(())
                }
            }
            TradeStatus::PartiallyOpen | TradeStatus::PendingCloseOrder => {
                if current_trade_status == TradeStatus::PartiallyOpen {
                    let mut open_order = current_trade.open_order.clone();
                    let left_units = open_order.get_executed_quantity() - open_order.units;
                    let updated_units = Some(left_units);
                    let updated_price = None;
                    let updated_stop_loss_price = None;
                    let updated_take_profit_price = None;
                    let amend_result = exchange
                        .amend_order(
                            current_trade.open_order.id.clone(),
                            updated_units,
                            updated_price,
                            updated_stop_loss_price,
                            updated_take_profit_price,
                        )
                        .await;
                    match amend_result {
                        Ok(amended) => {
                            if amended {
                                open_order.update_units(left_units);
                                current_trade = current_trade.update_trade(open_order)?;
                            } else {
                                let error = format!(
                                    "TradeStatus::PartiallyOpen -> amend order returned false"
                                );
                                let error = CustomError::new(error);
                                return Err(Error::CustomError(error));
                            }
                        }
                        Err(error) => {
                            let error = format!(
                                "TradeStatus::PartiallyOpen -> amend result failed! {:?}",
                                error
                            );
                            let error = Error::CustomError(CustomError::new(error));
                            return Err(error);
                        }
                    }
                }

                match exchange
                    .try_close_position(
                        &current_trade,
                        last_price,
                    )
                    .await
                {
                    Ok(close_order) => Ok(()),
                    Err(error) => {
                        let error = format!(
                            "TradeStatus::PartiallyOpen -> try close position result failed! {:?}",
                            error
                        );
                        let error = Error::CustomError(CustomError::new(error));
                        Err(error)
                    }
                }
            }
            _ => Ok(()),
        }
    } else {
        let available_to_withdraw = current_balance.value().available_to_withdraw;
        match signal {
            SignalCategory::GoLong => {
                open_order(
                    trading_settings,
                    exchange,
                    Side::Buy,
                    available_to_withdraw,
                    last_price,
                )
                .await
            }
            SignalCategory::GoShort => {
                open_order(
                    trading_settings,
                    exchange,
                    Side::Sell,
                    available_to_withdraw,
                    last_price,
                )
                .await
            }
            _ => Ok(()),
        }
    }
}

async fn open_order(
    trading_settings: TradingSettings,
    exchange: Box<dyn Exchange + Send + Sync>,
    side: Side,
    available_to_withdraw: f64,
    last_price: f64,
) -> Result<(), Error> {
    let stop_loss_percentage_opt = if let Some(modifier) = trading_settings
        .price_level_modifier_map
        .get(&PriceLevel::StopLoss(0.0).get_hash_key())
    {
        Some(modifier.get_percentage())
    } else {
        None
    };
    let take_profit_percentage_opt = if let Some(modifier) = trading_settings
        .price_level_modifier_map
        .get(&PriceLevel::TakeProfit(0.0).get_hash_key())
    {
        Some(modifier.get_percentage())
    } else {
        None
    };
    let leverage_factor = trading_settings.leverage.get_factor();

    let allocation = available_to_withdraw * trading_settings.allocation_percentage;

    match exchange
        .open_order(
            side,
            allocation,
            last_price,
        )
        .await
    {
        Ok(open_order) => Ok(()),
        Err(error) => {
            let error = format!(
                "Open order error. side {:?}, last price: {:?} {:?}",
                side, last_price, error
            );
            let error = Error::CustomError(CustomError::new(error));
            Err(error)
        }
    }
}

fn get_process_trading_data_handle(
    strategy_arc: Arc<Mutex<Strategy>>,
    performance_arc: Arc<Mutex<Performance>>,
    exchange_socket_error_arc: Arc<Mutex<Option<i64>>>,
    temp_executions_arc: Arc<Mutex<Vec<Execution>>>,
    trading_data_listener: BehaviorSubject<DataFrame>,
    trading_data_update_listener: BehaviorSubject<TradingDataUpdate>,
    exchange_listener: BehaviorSubject<Box<dyn Exchange + Send + Sync>>,
    current_trade_listener: BehaviorSubject<Option<Trade>>,
    update_balance_listener: BehaviorSubject<Option<Balance>>,
    update_order_listener: BehaviorSubject<Option<OrderAction>>,
    update_executions_listener: BehaviorSubject<Vec<Execution>>,
    current_balance_listener: BehaviorSubject<Balance>,
    signal_listener: BehaviorSubject<Option<SignalCategory>>,
) -> JoinHandle<()> {
    spawn(async move {
        let mut subscription = trading_data_update_listener.subscribe();
        while let Some(trading_data_update) = subscription.next().await {
            match trading_data_update {
                TradingDataUpdate::Nil => {}
                TradingDataUpdate::BenchmarkData {
                    initial_tick_data_lf,
                    initial_last_bar,
                } => {
                    let strategy_guard = strategy_arc.lock().unwrap();

                    let mut initial_trading_data_lf = strategy_guard
                        .set_benchmark(initial_tick_data_lf, initial_last_bar)
                        .expect(
                            "TradingDataUpdate::BenchmarkData -> strategy.set_benchmark.unwrap",
                        );

                    let mut performance_guard = performance_arc
                        .lock()
                        .expect("TradingDataUpdate::BenchmarkData -> performance_arc.unwrap");
                    let _ = performance_guard.set_benchmark_stats(&initial_trading_data_lf);
                    initial_trading_data_lf = initial_trading_data_lf.cache();
                    let initial_trading_data_df = initial_trading_data_lf.collect().expect(
                        "TradingDataUpdate::BenchmarkData -> performance_guard.set_benchmark.unwrap",
                    );
                    trading_data_listener.next(initial_trading_data_df);
                }
                TradingDataUpdate::MarketData {
                    last_period_tick_data,
                } => {
                    let current_trading_data = trading_data_listener.value();
                    let strategy_guard = strategy_arc.lock().unwrap();
                    let strategy_data = strategy_guard
                        .update_strategy_data(current_trading_data, last_period_tick_data)
                        .expect(
                            "TradingDataUpdate::MarketData -> strategy_guard.update_strategy_data",
                        );
                    // trading_data_listener.next(strategy_data);

                    let trading_data_update = TradingDataUpdate::StrategyData { strategy_data };

                    trading_data_update_listener.next(trading_data_update);
                }
                TradingDataUpdate::StrategyData { strategy_data } => {
                    // let strategy_data = trading_data_listener.value();
                    let exchange_socket_error_ts;
                    {
                        let exchange_socket_error_guard = exchange_socket_error_arc.lock().expect(
                            "TradingDataUpdate::StrategyData -> exchange_socket_error_guard.unwrap",
                        );
                        exchange_socket_error_ts = exchange_socket_error_guard.clone();
                    }

                    // checks for exchange ws error
                    if exchange_socket_error_ts.is_some() {
                        // in case of exchange ws error, this function fetches updates at this point and update listener accordingly
                        let _ = update_position_data_on_faulty_exchange_ws(
                            &exchange_socket_error_arc,
                            &exchange_listener,
                            &current_trade_listener,
                            &update_balance_listener,
                            &update_order_listener,
                            &update_executions_listener,
                        )
                        .await;
                    }

                    let trading_data = update_trading_data(
                        strategy_data,
                        &current_balance_listener,
                        &signal_listener,
                        &current_trade_listener,
                        &exchange_listener,
                    )
                    .expect("TradingDataUpdate::StrategyData -> update_trading_data unwrap failed");
                    trading_data_listener.next(trading_data.clone());
                    trading_data_update_listener
                        .next(TradingDataUpdate::EmitSignal { trading_data });
                }
                TradingDataUpdate::EmitSignal { trading_data } => {
                    // let trading_data = trading_data_listener.value();
                    let strategy_guard = strategy_arc.lock().unwrap();
                    let signal = strategy_guard
                        .generate_last_position_signal(&trading_data)
                        .expect(
                            "TradingDataUpdate::EmitSignal -> generate_last_position_signal unwrap",
                        );
                    // println!("emitted signal: {:?}", signal);
                    signal_listener.next(Some(signal));
                    trading_data_update_listener.next(TradingDataUpdate::CleanUp { trading_data });
                }
                TradingDataUpdate::CleanUp { trading_data } => {
                    // let trading_data = trading_data_listener.value();
                    let mut performance_guard = performance_arc
                        .lock()
                        .expect("TradingDataUpdate::CleanUp -> performance_arc.unwrap");
                    let _ = performance_guard.update_trading_stats(&trading_data);

                    let current_trade = current_trade_listener.value();

                    if let Some(current_trade) = current_trade {
                        let mut temp_executions_guard = temp_executions_arc
                            .lock()
                            .expect("TradingDataUpdate::CleanUp -> temp_executions deadlock");

                        let open_order_uuid = &current_trade.open_order.uuid;

                        let close_order_uuid =
                            &current_trade.close_order.clone().unwrap_or_default().uuid;

                        let mut pending_executions = vec![];
                        let mut removed_executions_ids = vec![];

                        while let Some(execution) = temp_executions_guard.iter().next() {
                            if &execution.order_uuid == open_order_uuid
                                || close_order_uuid != ""
                                    && &execution.order_uuid == close_order_uuid
                            {
                                pending_executions.push(execution.clone());
                                removed_executions_ids.push(execution.id.clone());
                            }
                        }

                        if pending_executions.len() > 0 {
                            let updated_trade = current_trade
                                .update_executions(pending_executions)
                                .expect("TradingDataUpdate::CleanUp update_executions unwrap");

                            if updated_trade.is_some() {
                                let updated_trade = updated_trade.unwrap();
                                current_trade_listener.next(Some(updated_trade));

                                let filtered_temp_executions = temp_executions_guard
                                    .clone()
                                    .into_iter()
                                    .filter(|execution| {
                                        !removed_executions_ids.contains(&execution.id)
                                    })
                                    .collect::<Vec<Execution>>();

                                *temp_executions_guard = filtered_temp_executions;
                            }
                        }
                    }
                }
            }
        }
    })
}

async fn get_update_balance_handle(
    update_balance_listener: BehaviorSubject<Option<Balance>>,
    current_balance_listener: BehaviorSubject<Balance>,
) -> JoinHandle<()> {
    spawn(async move {
        let mut subscription = update_balance_listener.subscribe();
        while let Some(balance_update) = subscription.next().await {
            match balance_update {
                Some(balance) => current_balance_listener.next(balance),
                None => {}
            }
        }
    })
}
// temp_executions_arc: Arc<Mutex<Vec<Execution>>>,
async fn get_update_order_handle(
    update_order_listener: BehaviorSubject<Option<OrderAction>>,
    current_trade_listener: BehaviorSubject<Option<Trade>>,
    temp_executions_arc: Arc<Mutex<Vec<Execution>>>,
    trading_settings_arc: Arc<Mutex<TradingSettings>>,
) -> JoinHandle<()> {
    spawn(async move {
        let mut subscription = update_order_listener.subscribe();
        while let Some(order_update) = subscription.next().await {
            match order_update {
                Some(order_action) => {
                    // println!("@@@ order_update");
                    match order_action.clone() {
                        OrderAction::Update(mut updated_order)
                        | OrderAction::Stop(mut updated_order) => {
                            updated_order = add_executions_to_order_and_remove_from_temp(
                                &temp_executions_arc,
                                updated_order,
                            );

                            // println!(
                            //     "updated order after pushing executions {:?}",
                            //     &updated_order
                            // );

                            let current_trade = current_trade_listener.value();
                            // println!("current trade {:?}", &current_trade);

                            if let Some(current_trade) = current_trade {
                                match current_trade.update_trade(updated_order.clone()) {
                                    Ok(updated_trade) => {
                                        // println!("match trade, updated {:?}", &updated_trade);
                                        if let OrderAction::Stop(_) = order_action {
                                            let (pnl, returns) =
                                                updated_trade.calculate_pnl_and_returns();
                                            let icon = if pnl > 0.0 { "📈" } else { "📉" };
                                            let warning = format!(
                                                "\n{:?} | {} Position {:?} was stopped. Profit and loss = {}, returns = {}",
                                                current_datetime(),
                                                icon,
                                                updated_trade.open_order.side,
                                                pnl,
                                                returns
                                            );
                                            println!("{}", warning);
                                        }

                                        // println!("updated_trade {:?}", &updated_trade);

                                        current_trade_listener.next(Some(updated_trade));
                                    }
                                    Err(error) => {
                                        println!(
                                            "OrderAction::Update | OrderAction::Stop -> error while updating -> error {:?}, trade = {:?}, order = {:?}",
                                            error,
                                            current_trade,
                                            updated_order,
                                        );
                                    }
                                }
                            } else {
                                if updated_order.is_stop {
                                    println!("OrderAction::Update | OrderAction::Stop -> received a stop order update with an empty trade");
                                    continue;
                                }

                                if updated_order.is_close {
                                    println!("OrderAction::Update | OrderAction::Stop -> received a close order update with an empty trade");
                                    continue;
                                }

                                let trading_settings_guard = trading_settings_arc
                                    .lock()
                                    .expect("trading_settings_arc unwrap");

                                println!(
                                    "\n{:?} | 📖 Opened {:?} order ({:?} units)",
                                    current_datetime(),
                                    updated_order.side,
                                    &updated_order.units,
                                );

                                let new_trade = Trade::new(
                                    updated_order,
                                    None,
                                );

                                current_trade_listener.next(Some(new_trade));
                            }
                        }
                        OrderAction::Cancel(cancelled_order) => {
                            let current_trade = current_trade_listener.value();
                            if let Some(current_trade) = current_trade {
                                // check if cancelled order is open order
                                if cancelled_order.id == current_trade.open_order.id {
                                    let cancelled_order = current_trade.open_order.cancel();
                                    let updated_trade = current_trade
                                        .update_trade(cancelled_order)
                                        .expect("OrderAction::Cancel -> update_trade unwrap");

                                    current_trade_listener.next(Some(updated_trade));
                                } else {
                                    println!(
                                        r#"
                                        OrderAction::Cancel -> cancelled order not related to current open order.
                                        Cancelled order = {:?}
                                        Current_trade = {:?}
                                        "#,
                                        cancelled_order, current_trade
                                    );
                                }
                            }
                        }
                    }
                }
                None => {}
            }
        }
    })
}

async fn get_update_executions_handle(
    update_executions_listener: BehaviorSubject<Vec<Execution>>,
    temp_executions_arc: Arc<Mutex<Vec<Execution>>>,
) -> JoinHandle<()> {
    spawn(async move {
        let mut subscription = update_executions_listener.subscribe();
        while let Some(latest_executions) = subscription.next().await {
            if latest_executions.len() == 0 {
                continue;
            }

            let mut temp_executions_guard = temp_executions_arc
                .lock()
                .expect("get_actions_handle -> temp_executions_guard deadlock");
            temp_executions_guard.extend(latest_executions);
            println!(
                "temp_executions_guard lenght {}",
                temp_executions_guard.len()
            );
        }
    })
}

fn add_executions_to_order_and_remove_from_temp(
    temp_executions_arc: &Arc<Mutex<Vec<Execution>>>,
    order: Order,
) -> Order {
    let mut updated_order = order.clone();
    let mut temp_executions_guard = temp_executions_arc
        .lock()
        .expect("process_last_signal -> temp_executions locked!");

    let order_uuid = &order.uuid;

    let mut pending_executions = vec![];
    let mut removed_executions_ids = vec![];

    let mut iterator = temp_executions_guard.iter();
    while let Some(execution) = iterator.next() {
        if &execution.order_uuid != "" && &execution.order_uuid == order_uuid {
            pending_executions.push(execution.clone());
            removed_executions_ids.push(execution.id.clone());
        }
    }

    if pending_executions.len() > 0 {
        updated_order = updated_order.push_executions_if_new(pending_executions);
        let filtered_temp_executions = temp_executions_guard
            .clone()
            .into_iter()
            .filter(|execution| !removed_executions_ids.contains(&execution.id))
            .collect::<Vec<Execution>>();

        *temp_executions_guard = filtered_temp_executions;
    }
    updated_order
}

fn update_trading_data(
    strategy_updated_data: DataFrame,
    current_balance_listener: &BehaviorSubject<Balance>,
    signal_listener: &BehaviorSubject<Option<SignalCategory>>,
    current_trade_listener: &BehaviorSubject<Option<Trade>>,
    exchange_listener: &BehaviorSubject<Box<dyn Exchange + Send + Sync>>,
) -> Result<DataFrame, Error> {
    println!("{} update_trading_data", current_timestamp_ms());
    // missing trade_fees, units, profit_and_loss, returns, balance, position, action
    let mut strategy_updated_data_clone = strategy_updated_data.clone();
    let series_binding = strategy_updated_data.columns([
        "start_time",
        "trade_fees",
        "units",
        "profit_and_loss",
        "returns",
        "balance",
        "position",
        "action",
    ])?;

    let mut series = series_binding.iter();

    let start_times_vec: Vec<Option<i64>> = series
        .next()
        .expect("update_trading_data -> start_time .next error")
        .datetime()
        .expect("update_trading_data -> start_time .f64 unwrap error")
        .into_iter()
        .collect();

    let mut trades_fees_vec: Vec<Option<f64>> = series
        .next()
        .expect("update_trading_data -> trades_fees_vec .next error")
        .f64()
        .expect("update_trading_data -> trades_fees_vec .f64 unwrap error")
        .into_iter()
        .collect();

    let mut units_vec: Vec<Option<f64>> = series
        .next()
        .expect("update_trading_data -> units_vec .next error")
        .f64()
        .expect("update_trading_data -> units_vec .f64 unwrap error")
        .into_iter()
        .collect();

    let mut pnl_vec: Vec<Option<f64>> = series
        .next()
        .expect("pnl_vec .next error")
        .f64()
        .expect("pnl_vec .f64 unwrap error")
        .into_iter()
        .collect();

    let mut returns_vec: Vec<Option<f64>> = series
        .next()
        .expect("returns_vec .next error")
        .f64()
        .expect("returns_vec .f64 unwrap error")
        .into_iter()
        .collect();

    let mut balances_vec: Vec<Option<f64>> = series
        .next()
        .expect("update_trading_data -> balances_vec .next error")
        .f64()
        .expect("update_trading_data -> balances_vec .f64 unwrap error")
        .into_iter()
        .collect();

    let mut positions_vec: Vec<Option<i32>> = series
        .next()
        .expect("update_trading_data -> positions_vec .next error")
        .i32()
        .expect("update_trading_data -> positions_vec .i32 unwrap error")
        .into_iter()
        .collect();

    let mut actions_vec: Vec<Option<&str>> = series
        .next()
        .expect("update_trading_data -> actions_vec .next error")
        .utf8()
        .expect("update_trading_data -> actions_vec .utf8 unwrap error")
        .into_iter()
        .collect();

    if start_times_vec.is_empty() {
        let error = CustomError {
            message: "start_times vector is empty".to_string(),
        };
        return Err(Error::from(error));
    }

    let index = start_times_vec.len() - 1;
    let previous_index = index - 1;

    // if previous_index < 0 {
    //     let error = format!(
    //         "update_trading_data -> penultimate index is less than 0 -> {:?}",
    //         &strategy_updated_data
    //     );
    //     return Err(Error::CustomError(CustomError::new(error)));
    // }

    let balance = current_balance_listener.value();
    balances_vec[index] = Some(balance.available_to_withdraw);
    let signal = signal_listener.value().unwrap_or_default();
    actions_vec[index] = Some(signal.get_column());
    let trade = current_trade_listener.value();

    let exchange_binding = exchange_listener.value();
    let traded_symbol = &exchange_binding.get_traded_contract().symbol;
    let close_col = get_symbol_close_col(traded_symbol);

    match trade {
        Some(current_trade) => {
            let trade_status = current_trade.status();
            match trade_status {
                TradeStatus::Cancelled | TradeStatus::Closed => {}
                _ => {
                    let current_price = &strategy_updated_data
                        .column(&close_col)
                        .expect("update_trading_data -> _ arm -> column unwrap")
                        .f64()
                        .expect("update_trading_data -> _ arm -> f64 unwrap")
                        .into_iter()
                        .last()
                        .expect("update_trading_data -> _ arm -> 1st option unwrap")
                        .expect("update_trading_data -> _ arm -> 2nd option unwrap");

                    let interval_start_timestamp = start_times_vec[previous_index]
                        .expect("update_trading_data -> _ arm -> interval_start_timestamp unwrap");
                    let interval_end_timestamp = start_times_vec[index]
                        .expect("update_trading_data -> _ arm -> interval_end_timestamp unwrap");

                    let (profit_and_loss, current_returns) = current_trade
                        .calculate_current_pnl_and_returns(interval_end_timestamp, *current_price);

                    let interval_fee = current_trade.get_executed_fees_between_interval(
                        interval_start_timestamp,
                        interval_end_timestamp,
                    );

                    let current_units = current_trade.get_interval_units(interval_end_timestamp);

                    trades_fees_vec[index] = Some(interval_fee);
                    units_vec[index] = Some(current_units);
                    pnl_vec[index] = Some(profit_and_loss);
                    returns_vec[index] = Some(current_returns);
                    positions_vec[index] = Some(current_trade.open_order.side.into());
                }
            }
        }
        None => {
            trades_fees_vec[index] = Some(0.0);
            units_vec[index] = Some(0.0);
            pnl_vec[index] = Some(0.0);
            returns_vec[index] = Some(0.0);
            positions_vec[index] = Some(0);
        }
    }

    // updates df
    strategy_updated_data_clone
        .replace("trade_fees", Series::new("trade_fees", trades_fees_vec))?;
    strategy_updated_data_clone.replace("units", Series::new("units", units_vec))?;
    strategy_updated_data_clone
        .replace("profit_and_loss", Series::new("profit_and_loss", pnl_vec))?;
    strategy_updated_data_clone.replace("returns", Series::new("returns", returns_vec))?;
    strategy_updated_data_clone.replace("balance", Series::new("balance", balances_vec))?;
    strategy_updated_data_clone.replace("position", Series::new("position", positions_vec))?;
    strategy_updated_data_clone.replace("action", Series::new("action", actions_vec))?;

    Ok(strategy_updated_data_clone)
}

fn on_close_update_trading_data(
    strategy_updated_data: DataFrame,
    current_balance_listener: &BehaviorSubject<Balance>,
    signal_listener: &BehaviorSubject<Option<SignalCategory>>,
    current_trade_listener: &BehaviorSubject<Option<Trade>>,
) -> Result<Option<DataFrame>, Error> {
    println!("{} on_close_update_trading_data", current_timestamp_ms());
    // missing trade_fees, units, profit_and_loss, returns, balance, position, action
    let mut strategy_updated_data_clone = strategy_updated_data.clone();
    let series_binding = strategy_updated_data.columns([
        "start_time",
        "trade_fees",
        "units",
        "profit_and_loss",
        "returns",
        "balance",
        "position",
        "action",
    ])?;

    let mut series = series_binding.iter();

    let start_times_vec: Vec<Option<i64>> = series
        .next()
        .expect("on_close_update_trading_data -> start_time .next error")
        .datetime()
        .expect("on_close_update_trading_data -> start_time .f64 unwrap error")
        .into_iter()
        .collect();

    let mut trades_fees_vec: Vec<Option<f64>> = series
        .next()
        .expect("on_close_update_trading_data -> trades_fees_vec .next error")
        .f64()
        .expect("on_close_update_trading_data -> trades_fees_vec .f64 unwrap error")
        .into_iter()
        .collect();

    let mut units_vec: Vec<Option<f64>> = series
        .next()
        .expect("on_close_update_trading_data -> units_vec .next error")
        .f64()
        .expect("on_close_update_trading_data -> units_vec .f64 unwrap error")
        .into_iter()
        .collect();

    let mut pnl_vec: Vec<Option<f64>> = series
        .next()
        .expect("on_close_update_trading_data pnl_vec .next error")
        .f64()
        .expect("on_close_update_trading_data pnl_vec .f64 unwrap error")
        .into_iter()
        .collect();

    let mut returns_vec: Vec<Option<f64>> = series
        .next()
        .expect("on_close_update_trading_data returns_vec .next error")
        .f64()
        .expect("on_close_update_trading_data returns_vec .f64 unwrap error")
        .into_iter()
        .collect();

    let mut balances_vec: Vec<Option<f64>> = series
        .next()
        .expect("on_close_update_trading_data -> balances_vec .next error")
        .f64()
        .expect("on_close_update_trading_data -> balances_vec .f64 unwrap error")
        .into_iter()
        .collect();

    let mut positions_vec: Vec<Option<i32>> = series
        .next()
        .expect("on_close_update_trading_data -> positions_vec .next error")
        .i32()
        .expect("on_close_update_trading_data -> positions_vec .i32 unwrap error")
        .into_iter()
        .collect();

    let mut actions_vec: Vec<Option<&str>> = series
        .next()
        .expect("on_close_update_trading_data -> actions_vec .next error")
        .utf8()
        .expect("on_close_update_trading_data -> actions_vec .utf8 unwrap error")
        .into_iter()
        .collect();

    if start_times_vec.is_empty() {
        let error = CustomError {
            message: "on_close_update_trading_data start_times vector is empty".to_string(),
        };
        return Err(Error::from(error));
    }

    let index = start_times_vec.len() - 1;

    let balance = current_balance_listener.value();
    balances_vec[index] = Some(balance.available_to_withdraw);
    let signal = signal_listener.value().unwrap_or_default();
    actions_vec[index] = Some(signal.get_column());
    let trade = current_trade_listener.value();

    let mut result = None;
    match trade {
        Some(current_trade) => {
            let trade_status = current_trade.status();
            match trade_status {
                TradeStatus::Cancelled | TradeStatus::Closed => {
                    if trade_status == TradeStatus::Cancelled {
                        trades_fees_vec[index] = Some(0.0);
                        units_vec[index] = Some(0.0);
                        pnl_vec[index] = Some(0.0);
                        returns_vec[index] = Some(0.0);
                        positions_vec[index] = Some(0);
                    } else {
                        let interval_start_timestamp =  start_times_vec[index]
                                .expect("update_trading_data -> TradeStatus::Closed arm -> interval_start_timestamp unwrap");
                        let interval_end_timestamp = current_timestamp_ms() as i64;

                        let (profit_and_loss, current_returns) =
                            current_trade.calculate_pnl_and_returns();

                        let interval_fee = current_trade.get_executed_fees_between_interval(
                            interval_start_timestamp,
                            interval_end_timestamp,
                        );

                        trades_fees_vec[index] = Some(interval_fee);
                        units_vec[index] = Some(0.0);
                        pnl_vec[index] = Some(profit_and_loss);
                        returns_vec[index] = Some(current_returns);
                        positions_vec[index] = Some(0);
                    }

                    // updates df
                    strategy_updated_data_clone
                        .replace("trade_fees", Series::new("trade_fees", trades_fees_vec))?;
                    strategy_updated_data_clone
                        .replace("units", Series::new("units", units_vec))?;
                    strategy_updated_data_clone
                        .replace("profit_and_loss", Series::new("profit_and_loss", pnl_vec))?;
                    strategy_updated_data_clone
                        .replace("returns", Series::new("returns", returns_vec))?;
                    strategy_updated_data_clone
                        .replace("balance", Series::new("balance", balances_vec))?;
                    strategy_updated_data_clone
                        .replace("position", Series::new("position", positions_vec))?;
                    strategy_updated_data_clone
                        .replace("action", Series::new("action", actions_vec))?;
                    result = Some(strategy_updated_data_clone);
                }
                _ => {}
            }
        }
        None => {}
    }

    Ok(result)
}
