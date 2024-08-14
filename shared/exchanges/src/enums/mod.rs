use std::{
    sync::{Arc, Mutex, RwLock},
    time::Duration,
};

use chrono::NaiveDateTime;
use common::{
    enums::{
        balance::Balance, modifiers::leverage::Leverage, order_action::OrderAction,
        order_status::OrderStatus, order_type::OrderType, side::Side, trade_status::TradeStatus,
        trading_data_update::TradingDataUpdate,
    },
    structs::{BehaviorSubject, Contract, Execution, Order, Trade, TradingSettings},
    traits::exchange::{BenchmarkExchange, DataProviderExchange, TraderExchange, TraderHelper},
};
use glow_error::GlowError;
use polars::prelude::Schema;
use reqwest::Client;
use tokio::net::TcpStream;
use tokio_tungstenite::{tungstenite::Message, MaybeTlsStream, WebSocketStream};
use url::Url;

use crate::{binance::structs::BinanceDataProvider, bybit::BybitTraderExchange};

#[derive(Clone)]
pub enum DataProviderExchangeWrapper {
    Binance(BinanceDataProvider),
}

impl DataProviderExchangeWrapper {
    pub fn new_binance_data_provider(
        kline_duration: Duration,
        last_ws_error_ts: &Arc<Mutex<Option<i64>>>,
        minimum_klines_for_benchmarking: u32,
        symbols: (&'static str, &'static str),
        trading_data_update_listener: &'static BehaviorSubject<TradingDataUpdate>,
    ) -> Self {
        Self::Binance(BinanceDataProvider::new(
            kline_duration,
            last_ws_error_ts,
            minimum_klines_for_benchmarking,
            symbols,
            trading_data_update_listener,
        ))
    }

    pub fn get_selection_list() -> Vec<String> {
        vec![String::from("Binance")]
    }
}

impl DataProviderExchange for DataProviderExchangeWrapper {
    async fn subscribe_to_tick_stream(
        &mut self,
        wss: &mut WebSocketStream<MaybeTlsStream<TcpStream>>,
    ) -> Result<(), GlowError> {
        match self {
            Self::Binance(ex) => ex.subscribe_to_tick_stream(wss).await,
        }
    }

    async fn listen_ticks(
        &mut self,
        wss: WebSocketStream<MaybeTlsStream<TcpStream>>,
        benchmark_end: NaiveDateTime,
        trading_data_schema: &Schema,
    ) -> Result<(), GlowError> {
        match self {
            Self::Binance(ex) => {
                ex.listen_ticks(wss, benchmark_end, trading_data_schema)
                    .await
            }
        }
    }

    async fn init(
        &mut self,
        benchmark_end: NaiveDateTime,
        benchmark_start: Option<NaiveDateTime>,
        kline_data_schema: Schema,
        run_benchmark_only: bool,
        trading_data_schema: Schema,
    ) -> Result<(), GlowError> {
        match self {
            Self::Binance(ex) => {
                ex.init(
                    benchmark_end,
                    benchmark_start,
                    kline_data_schema,
                    run_benchmark_only,
                    trading_data_schema,
                )
                .await
            }
        }
    }
}
#[derive(Clone)]
pub enum TraderExchangeWrapper {
    Bybit(BybitTraderExchange),
}

impl TraderExchangeWrapper {
    pub fn new_bybit_trader(
        current_trade_listener: &BehaviorSubject<Option<Trade>>,
        last_ws_error_ts: &Arc<Mutex<Option<i64>>>,
        trading_settings: &Arc<RwLock<TradingSettings>>,
        update_balance_listener: &BehaviorSubject<Option<Balance>>,
        update_executions_listener: &BehaviorSubject<Vec<Execution>>,
        update_order_listener: &BehaviorSubject<Option<OrderAction>>,
    ) -> Self {
        Self::Bybit(BybitTraderExchange::new(
            current_trade_listener,
            last_ws_error_ts,
            trading_settings,
            update_balance_listener,
            update_executions_listener,
            update_order_listener,
        ))
    }

    pub fn get_selection_list() -> Vec<String> {
        vec![String::from("Bybit")]
    }
}

impl TraderHelper for TraderExchangeWrapper {
    fn get_trading_settings(&self) -> TradingSettings {
        match self {
            Self::Bybit(ex) => ex.get_trading_settings(),
        }
    }

    fn get_taker_fee(&self) -> f64 {
        match self {
            Self::Bybit(ex) => ex.get_taker_fee(),
        }
    }

    fn get_maker_fee(&self) -> f64 {
        match self {
            Self::Bybit(ex) => ex.get_maker_fee(),
        }
    }

    fn calculate_open_order_units_and_balance_remainder(
        &self,
        side: Side,
        order_cost: f64,
        price: f64,
    ) -> Result<(f64, f64), GlowError> {
        match self {
            Self::Bybit(ex) => {
                ex.calculate_open_order_units_and_balance_remainder(side, order_cost, price)
            }
        }
    }

    fn get_order_fee_rate(&self, order_type: OrderType) -> (f64, bool) {
        match self {
            Self::Bybit(ex) => ex.get_order_fee_rate(order_type),
        }
    }

    fn calculate_order_fees(
        &self,
        order_type: OrderType,
        side: Side,
        units: f64,
        price: f64,
    ) -> ((f64, f64), f64, bool) {
        match self {
            Self::Bybit(ex) => ex.calculate_order_fees(order_type, side, units, price),
        }
    }

    fn calculate_order_stop_loss_price(&self, side: Side, price: f64) -> Option<f64> {
        match self {
            Self::Bybit(ex) => ex.calculate_order_stop_loss_price(side, price),
        }
    }

    fn calculate_order_take_profit_price(&self, side: Side, price: f64) -> Option<f64> {
        match self {
            Self::Bybit(ex) => ex.calculate_order_take_profit_price(side, price),
        }
    }

    fn get_contracts(&self) -> &std::collections::HashMap<&str, Contract> {
        match self {
            Self::Bybit(ex) => ex.get_contracts(),
        }
    }
}

impl TraderExchange for TraderExchangeWrapper {
    fn new_open_order(&self, side: Side, order_cost: f64, price: f64) -> Result<Order, GlowError> {
        match self {
            Self::Bybit(ex) => ex.new_open_order(side, order_cost, price),
        }
    }

    fn get_ws_url(&self) -> Result<Url, GlowError> {
        match self {
            Self::Bybit(ex) => ex.get_ws_url(),
        }
    }

    async fn auth_ws(
        &self,
        wss: &mut WebSocketStream<MaybeTlsStream<TcpStream>>,
    ) -> Result<(), GlowError> {
        match self {
            Self::Bybit(ex) => ex.auth_ws(wss).await,
        }
    }

    async fn subscribe_ws(
        &self,
        wss: &mut WebSocketStream<MaybeTlsStream<TcpStream>>,
    ) -> Result<(), GlowError> {
        match self {
            Self::Bybit(ex) => ex.subscribe_ws(wss).await,
        }
    }

    async fn fetch_order_executions(
        &self,
        order_uuid: String,
        start_timestamp: i64,
        end_timestamp: i64,
    ) -> Result<Vec<Execution>, GlowError> {
        match self {
            Self::Bybit(ex) => {
                ex.fetch_order_executions(order_uuid, start_timestamp, end_timestamp)
                    .await
            }
        }
    }

    async fn fetch_history_order(
        &self,
        id: Option<String>,
        side: Option<Side>,
        fetch_executions: bool,
    ) -> Result<Order, GlowError> {
        match self {
            Self::Bybit(ex) => ex.fetch_history_order(id, side, fetch_executions).await,
        }
    }

    async fn fetch_current_order(
        &self,
        order_id: String,
        fetch_executions: bool,
    ) -> Result<Order, GlowError> {
        match self {
            Self::Bybit(ex) => ex.fetch_current_order(order_id, fetch_executions).await,
        }
    }

    async fn fetch_current_trade_position(&self) -> Result<Option<Trade>, GlowError> {
        match self {
            Self::Bybit(ex) => ex.fetch_current_trade_position().await,
        }
    }

    async fn fetch_trade_state(
        &self,
        trade_id: String,
        last_status: TradeStatus,
    ) -> Result<Trade, GlowError> {
        match self {
            Self::Bybit(ex) => ex.fetch_trade_state(trade_id, last_status).await,
        }
    }

    async fn fetch_current_usdt_balance(&self) -> Result<Balance, GlowError> {
        match self {
            Self::Bybit(ex) => ex.fetch_current_usdt_balance().await,
        }
    }

    async fn open_order(
        &self,
        side: Side,
        amount: f64,
        expected_price: f64,
    ) -> Result<Order, GlowError> {
        match self {
            Self::Bybit(ex) => ex.open_order(side, amount, expected_price).await,
        }
    }

    async fn amend_order(
        &self,
        order_id: String,
        updated_units: Option<f64>,
        updated_price: Option<f64>,
        updated_stop_loss_price: Option<f64>,
        updated_take_profit_price: Option<f64>,
    ) -> Result<bool, GlowError> {
        match self {
            Self::Bybit(ex) => {
                ex.amend_order(
                    order_id,
                    updated_units,
                    updated_price,
                    updated_stop_loss_price,
                    updated_take_profit_price,
                )
                .await
            }
        }
    }

    async fn try_close_position(&self, trade: &Trade, est_price: f64) -> Result<Order, GlowError> {
        match self {
            Self::Bybit(ex) => ex.try_close_position(trade, est_price).await,
        }
    }

    async fn cancel_order(&self, order_id: String) -> Result<bool, GlowError> {
        match self {
            Self::Bybit(ex) => ex.cancel_order(order_id).await,
        }
    }

    async fn set_leverage(&self, leverage: Leverage) -> Result<bool, GlowError> {
        match self {
            Self::Bybit(ex) => ex.set_leverage(leverage).await,
        }
    }

    fn get_http_client(&self) -> &Client {
        match self {
            Self::Bybit(ex) => ex.get_http_client(),
        }
    }

    fn get_ws_ping_interval(&self) -> u64 {
        match self {
            Self::Bybit(ex) => ex.get_ws_ping_interval(),
        }
    }

    fn get_ws_ping_message(&self) -> Result<Message, GlowError> {
        match self {
            Self::Bybit(ex) => ex.get_ws_ping_message(),
        }
    }

    fn process_ws_message(&self, json: &String) -> Result<(), GlowError> {
        match self {
            Self::Bybit(ex) => ex.process_ws_message(json),
        }
    }

    async fn update_position_data_on_faulty_exchange_ws(&self) -> Result<(), GlowError> {
        match self {
            Self::Bybit(ex) => ex.update_position_data_on_faulty_exchange_ws().await,
        }
    }

    async fn init(&mut self) -> Result<(), GlowError> {
        match self {
            Self::Bybit(ex) => ex.init().await,
        }
    }

    async fn listen_messages(
        &mut self,
        wss: WebSocketStream<MaybeTlsStream<TcpStream>>,
    ) -> Result<(), GlowError> {
        match self {
            Self::Bybit(ex) => ex.listen_messages(wss).await,
        }
    }
}

impl BenchmarkExchange for TraderExchangeWrapper {
    fn new_benchmark_open_order(
        &self,
        timestamp: i64,
        side: Side,
        order_cost: f64,
        price: f64,
    ) -> Result<Order, GlowError> {
        match self {
            Self::Bybit(ex) => ex.new_benchmark_open_order(timestamp, side, order_cost, price),
        }
    }

    fn new_benchmark_close_order(
        &self,
        timestamp: i64,
        trade_id: &String,
        close_price: f64,
        open_order: Order,
        final_status: OrderStatus,
    ) -> Result<Order, GlowError> {
        match self {
            Self::Bybit(ex) => ex.new_benchmark_close_order(
                timestamp,
                trade_id,
                close_price,
                open_order,
                final_status,
            ),
        }
    }

    fn check_price_level_modifiers(
        &self,
        trade: &Trade,
        current_timestamp: i64,
        close_price: f64,
        stop_loss: Option<&common::enums::modifiers::price_level::PriceLevel>,
        take_profit: Option<&common::enums::modifiers::price_level::PriceLevel>,
        trailing_stop_loss: Option<&common::enums::modifiers::price_level::PriceLevel>,
        current_peak_returns: f64,
    ) -> Result<Option<Trade>, GlowError> {
        match self {
            Self::Bybit(ex) => ex.check_price_level_modifiers(
                trade,
                current_timestamp,
                close_price,
                stop_loss,
                take_profit,
                trailing_stop_loss,
                current_peak_returns,
            ),
        }
    }
}