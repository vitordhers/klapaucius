fn compute_benchmark_positions() {
    // fn compute_benchmark_positions(
    //     &self,
    //     initial_strategy_df: DataFrame,
    // ) -> Result<DataFrame, GlowError> {
     // // let data = data.to_owned();
        // // TODO: TRY TO IMPLEMENT THIS USING LAZYFRAMES
        // let perf_start = Instant::now();

        // let mut df = initial_strategy_df;
        // let df_height = df.height();

        // let start_timestamps = df
        //     .column("start_time")
        //     .unwrap()
        //     .datetime()
        //     .unwrap()
        //     .into_no_null_iter()
        //     .collect::<Vec<i64>>();

        // let end_timestamps = start_timestamps
        //     .clone()
        //     .into_iter()
        //     .map(|start_timestamp| start_timestamp - 1)
        //     .collect::<Vec<i64>>();

        // let traded_symbol = self.trader_exchange.get_traded_symbol();
        // let (opens, highs, lows, closes) = get_price_columns(&df, &traded_symbol)?;
        // let shorts = get_signal_col_values(&df, SignalCategory::GoShort)?;
        // let longs = get_signal_col_values(&df, SignalCategory::GoShort)?;
        // let close_shorts = get_signal_col_values(&df, SignalCategory::CloseShort)?;
        // let close_longs = get_signal_col_values(&df, SignalCategory::CloseLong)?;

        // let mut trade_fees = vec![0.0];
        // let mut units = vec![0.0];
        // let mut profit_and_loss = vec![0.0];
        // let mut returns = vec![0.0];
        // let mut balances = vec![100.0];
        // let mut positions = vec![0];
        // let mut actions = vec![SignalCategory::KeepPosition.get_column().to_owned()];

        // let trading_settings = self.trader_exchange.get_trading_settings();

        // let leverage_factor = self.trader_exchange.get_leverage_factor();
        // let has_leverage = leverage_factor > 1.0;

        // let price_level_modifier_map_binding = trading_settings.price_level_modifier_map.clone();
        // let stop_loss: Option<&PriceLevel> = price_level_modifier_map_binding.get("sl");
        // let take_profit = price_level_modifier_map_binding.get("tp");
        // let trailing_stop_loss = price_level_modifier_map_binding.get("tsl");
        // let should_check_price_modifiers = has_leverage
        //     || stop_loss.is_some()
        //     || take_profit.is_some()
        //     || trailing_stop_loss.is_some();

        // let mut current_trade: Option<Trade> = None;
        // // let mut current_peak_returns = 0.0;
        // let mut current_min_price_threshold = None;
        // let mut current_max_price_threshold = None;

        // // need to be updated
        // // trade_fees, units, profit_and_loss, returns, balances, positions, actions

        // for index in 0..df_height {
        //     if index == 0 {
        //         continue;
        //     }

        //     let current_position = positions[index - 1];
        //     let current_units = units[index - 1];
        //     let current_balance = balances[index - 1];

        //     let default_results = (
        //         0.0,
        //         current_units,
        //         0.0,
        //         0.0,
        //         current_balance,
        //         current_position,
        //         SignalCategory::KeepPosition.get_column().to_owned(),
        //     );

        //     let (fee, unit, pnl, trade_returns, balance, position, action) = if current_position
        //         == 0
        //     {
        //         let should_short = shorts[index - 1] == 1;
        //         let should_long = longs[index - 1] == 1;
        //         if should_short || should_long {
        //             let start_timestamp = start_timestamps[index];
        //             let end_timestamp = end_timestamps[index];
        //             let open_price = opens[index];
        //             let close_price = closes[index];

        //             match self.trader_exchange.new_benchmark_open_order(
        //                 start_timestamp,
        //                 if should_short { Side::Sell } else { Side::Buy },
        //                 current_balance,
        //                 open_price,
        //             ) {
        //                 Ok(open_order) => {
        //                     let open_trade: Trade = open_order.clone().into();
        //                     let fees = open_trade.get_executed_fees();
        //                     let (pnl, trade_returns) = open_trade
        //                         .calculate_current_pnl_and_returns(end_timestamp, close_price);
        //                     let open_order_cost =
        //                         open_order.get_order_cost().expect("order to have cost");
        //                     (current_min_price_threshold, current_max_price_threshold) =
        //                         open_trade.get_threshold_prices();
        //                     current_trade = Some(open_trade);
        //                     (
        //                         fees,
        //                         open_order.units,
        //                         pnl,
        //                         trade_returns,
        //                         f64::max(0.0, current_balance - open_order_cost),
        //                         open_order.side.into(),
        //                         (if should_short {
        //                             SignalCategory::GoShort
        //                         } else {
        //                             SignalCategory::GoLong
        //                         })
        //                         .get_column()
        //                         .to_owned(),
        //                     )
        //                 }
        //                 Err(error) => {
        //                     println!("create_new_benchmark_open_order error {:?}", error);
        //                     default_results
        //                 }
        //             }
        //         } else {
        //             default_results
        //         }
        //     } else {
        //         let trade = current_trade.clone().unwrap();
        //         let current_side = trade.open_order.side;
        //         let stopped_result = if should_check_price_modifiers {
        //             let min_price = lows[index];
        //             let max_price = highs[index];
        //             let binds_on_min_price =
        //                 min_price <= current_min_price_threshold.unwrap_or_default();
        //             let binds_on_max_price =
        //                 max_price >= current_max_price_threshold.unwrap_or_default();
        //             if binds_on_min_price || binds_on_max_price {
        //                 // let prev_close_price = closes[index - 1];
        //                 let prev_end_timestamp = end_timestamps[index - 1];
        //                 match self.trader_exchange.close_benchmark_trade_on_binding_price(
        //                     &trade,
        //                     prev_end_timestamp,
        //                     if binds_on_min_price {
        //                         current_min_price_threshold.unwrap()
        //                     } else {
        //                         current_max_price_threshold.unwrap()
        //                     },
        //                 ) {
        //                     Ok(closed_trade) => {
        //                         let close_order = closed_trade.clone().close_order.unwrap();
        //                         let (pnl, trade_returns) = closed_trade.calculate_pnl_and_returns();
        //                         let order_cost = closed_trade.open_order.get_order_cost().unwrap();
        //                         let action = match close_order.status {
        //                             OrderStatus::StoppedBR => SignalCategory::LeverageBankrupcty,
        //                             OrderStatus::StoppedSL => SignalCategory::StopLoss,
        //                             OrderStatus::StoppedTP => SignalCategory::TakeProfit,
        //                             // OrderStatus::StoppedTSL => {
        //                             //     SignalCategory::TrailingStopLoss
        //                             // }
        //                             _ => SignalCategory::KeepPosition,
        //                         };
        //                         (current_min_price_threshold, current_max_price_threshold) =
        //                             (None, None);
        //                         current_trade = None;
        //                         Some((
        //                             close_order.get_executed_order_fee(),
        //                             0.0,
        //                             pnl,
        //                             trade_returns,
        //                             current_balance + order_cost + pnl,
        //                             0,
        //                             action.get_column().to_owned(),
        //                         ))
        //                     }
        //                     Err(error) => {
        //                         println!(
        //                             "close_benchmark_trade_on_binding_price error {:?}",
        //                             error
        //                         );
        //                         None
        //                     }
        //                 }
        //             } else {
        //                 None
        //             }
        //         } else {
        //             None
        //         };

        //         if let Some(stopped_result) = stopped_result {
        //             stopped_result
        //         } else {
        //             // position wasn't stopped
        //             let was_short_closed =
        //                 close_shorts[index - 1] == 1 && current_side == Side::Sell;
        //             let was_long_closed = close_longs[index - 1] == 1 && current_side == Side::Buy;

        //             if was_short_closed || was_long_closed {
        //                 let current_timestamp = start_timestamps[index];
        //                 let open_price = opens[index];

        //                 match self.trader_exchange.new_benchmark_close_order(
        //                     current_timestamp,
        //                     &trade.id,
        //                     open_price,
        //                     trade.open_order.clone(),
        //                     OrderStatus::Closed,
        //                 ) {
        //                     Ok(close_order) => {
        //                         let updated_trade = trade.update_trade(close_order.clone())?;
        //                         let (pnl, trade_returns) =
        //                             updated_trade.calculate_pnl_and_returns();
        //                         let order_cost = trade.open_order.get_order_cost().unwrap();

        //                         current_trade = None;
        //                         (
        //                             close_order.get_executed_order_fee(),
        //                             0.0,
        //                             pnl,
        //                             trade_returns,
        //                             current_balance + order_cost + pnl,
        //                             0,
        //                             (if was_short_closed {
        //                                 SignalCategory::CloseShort
        //                             } else {
        //                                 SignalCategory::CloseLong
        //                             })
        //                             .get_column()
        //                             .to_owned(),
        //                         )
        //                     }
        //                     Err(error) => {
        //                         println!("create_benchmark_close_order WARNING: {:?}", error);
        //                         default_results
        //                     }
        //                 }
        //             } else {
        //                 default_results
        //             }
        //         }
        //     };

        //     trade_fees.push(fee);
        //     units.push(unit);
        //     profit_and_loss.push(pnl);
        //     returns.push(trade_returns);
        //     balances.push(balance);
        //     positions.push(position);
        //     actions.push(action);
        // }
        // // if last position was taken
        // if positions.last().unwrap() != &0 {
        //     if let Some((before_last_order_index, _)) =
        //         positions // over positions vector
        //             .iter() // iterate over
        //             .enumerate() // an enumeration
        //             .rev() // of reversed positions
        //             .find(|(_, value)| value == &&0)
        //     // until it finds where value is 0
        //     {
        //         // splices results vectors to values before opening the order
        //         // note that even though the vector was reversed, before_last_order_index keeps being the original vector index. Thanks, Rust <3
        //         let range = before_last_order_index..df_height;
        //         let zeroed_float_patch: Vec<f64> = range.clone().map(|_| 0.0 as f64).collect();
        //         let zeroed_integer_patch: Vec<i32> = range.clone().map(|_| 0 as i32).collect();
        //         let keep_position_action_patch: Vec<String> = range
        //             .clone()
        //             .map(|_| SignalCategory::KeepPosition.get_column().to_owned())
        //             .collect();

        //         trade_fees.splice(range.clone(), zeroed_float_patch.clone());
        //         units.splice(range.clone(), zeroed_float_patch.clone());
        //         profit_and_loss.splice(range.clone(), zeroed_float_patch.clone());

        //         positions.splice(range.clone(), zeroed_integer_patch);
        //         actions.splice(range.clone(), keep_position_action_patch);

        //         let previous_balance = balances[before_last_order_index];
        //         let patch_balances: Vec<f64> =
        //             range.clone().map(|_| previous_balance as f64).collect();
        //         balances.splice(range.clone(), patch_balances);
        //         returns.splice(range.clone(), zeroed_float_patch);
        //     }
        // }

        // let trade_fee_series = Series::new("trade_fees", trade_fees);
        // let units_series = Series::new("units", units);
        // let profit_and_loss_series = Series::new("profit_and_loss", profit_and_loss);
        // let returns_series = Series::new("returns", returns);
        // let balance_series = Series::new("balance", balances);
        // let position_series = Series::new("position", positions);
        // let action_series = Series::new("action", actions);

        // let df = df.with_column(trade_fee_series)?;
        // let df = df.with_column(units_series)?;
        // let df = df.with_column(profit_and_loss_series)?;
        // let df = df.with_column(returns_series)?;
        // let df = df.with_column(balance_series)?;
        // let df = df.with_column(position_series)?;
        // let df = df.with_column(action_series)?;

        // let elapsed_time = perf_start.elapsed();
        // let elapsed_millis = elapsed_time.as_nanos();
        // println!(
        //     "compute_benchmark_positions => Elapsed time in nanos: {}",
        //     elapsed_millis
        // );

        // Ok(df.clone())
}