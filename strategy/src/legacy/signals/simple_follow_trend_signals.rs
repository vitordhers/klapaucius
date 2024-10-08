use common::{
    enums::signal_category::SignalCategory,
    structs::{Symbol, SymbolsPair},
    traits::signal::Signal,
};
use glow_error::GlowError;
use polars::prelude::*;

use super::SignalWrapper;

#[derive(Clone, Debug)]
pub struct SimpleFollowTrendShortSignal {
    pub anchor_symbol: &'static Symbol,
}

#[derive(Clone, Debug)]
pub struct SimpleFollowTrendLongSignal {
    pub anchor_symbol: &'static Symbol,
}

#[derive(Clone, Debug)]
pub struct SimpleFollowTrendCloseLongSignal {
    pub anchor_symbol: &'static Symbol,
}

#[derive(Clone, Debug)]
pub struct SimpleFollowTrendCloseShortSignal {
    pub anchor_symbol: &'static Symbol,
}

impl Signal for SimpleFollowTrendShortSignal {
    type Wrapper = SignalWrapper;
    fn signal_category(&self) -> SignalCategory {
        SignalCategory::GoShort
    }

    fn set_signal_column(&self, lf: &LazyFrame) -> Result<LazyFrame, GlowError> {
        let signal = self.signal_category();
        let signal_col = signal.get_column();
        let select_columns = vec![col("start_time"), col(signal_col)];
        let fast_ema_col_title = format!("{}_fast_ema", &self.anchor_symbol.name);
        let slow_ema_col_title = format!("{}_slow_ema", &self.anchor_symbol.name);
        let signal_lf = lf
            .clone()
            .with_column(
                when(
                    col(&fast_ema_col_title).lt(col(&slow_ema_col_title)).and(
                        col(&slow_ema_col_title)
                            .shift(1)
                            .lt_eq(col(&fast_ema_col_title).shift(1)),
                    ),
                )
                .then(lit(1))
                .otherwise(lit(0))
                .alias(signal_col),
            )
            .select(select_columns);
        Ok(signal_lf)
    }

    fn update_signal_column(&self, data: &DataFrame) -> Result<DataFrame, GlowError> {
        let mut new_lf = data.clone().lazy();
        new_lf = self.set_signal_column(&new_lf)?;
        let new_df = new_lf.collect()?;
        let mut result_df = data.clone();
        let signal = self.signal_category();
        let column = signal.get_column();
        let series = new_df.column(column)?;
        let _ = result_df.replace(&column, series.to_owned());

        Ok(result_df)
    }

    fn patch_symbols_pair(
        &self,
        updated_symbols_pair: SymbolsPair,
    ) -> Result<Self::Wrapper, GlowError> {
        if self.anchor_symbol == updated_symbols_pair.anchor {
            return Ok(self.clone().into());
        }
        let updated = Self {
            anchor_symbol: updated_symbols_pair.anchor,
        };
        Ok(updated.into())
    }
}

impl Signal for SimpleFollowTrendLongSignal {
    type Wrapper = SignalWrapper;
    fn signal_category(&self) -> SignalCategory {
        SignalCategory::GoLong
    }

    fn set_signal_column(&self, lf: &LazyFrame) -> Result<LazyFrame, GlowError> {
        let signal = self.signal_category();
        let signal_col = signal.get_column();
        let select_columns = vec![col("start_time"), col(signal_col)];
        let fast_ema_col_title = format!("{}_fast_ema", &self.anchor_symbol.name);
        let slow_ema_col_title = format!("{}_slow_ema", &self.anchor_symbol.name);
        let signal_lf = lf
            .clone()
            .with_column(
                when(
                    col(&fast_ema_col_title).gt(col(&slow_ema_col_title)).and(
                        col(&slow_ema_col_title)
                            .shift(1)
                            .gt_eq(col(&fast_ema_col_title).shift(1)),
                    ),
                )
                .then(lit(1))
                .otherwise(lit(0))
                .alias(signal_col),
            )
            .select(select_columns);
        Ok(signal_lf)
    }

    fn update_signal_column(&self, data: &DataFrame) -> Result<DataFrame, GlowError> {
        let mut new_lf = data.clone().lazy();
        new_lf = self.set_signal_column(&new_lf)?;
        let new_df = new_lf.collect()?;
        let mut result_df = data.clone();
        let signal = self.signal_category();
        let column = signal.get_column();
        let series = new_df.column(column)?;
        let _ = result_df.replace(&column, series.to_owned());

        Ok(result_df)
    }
    fn patch_symbols_pair(
        &self,
        updated_symbols_pair: SymbolsPair,
    ) -> Result<Self::Wrapper, GlowError> {
        if self.anchor_symbol == updated_symbols_pair.anchor {
            return Ok(self.clone().into());
        }
        let updated = Self {
            anchor_symbol: updated_symbols_pair.anchor,
        };
        Ok(updated.into())
    }
}

impl Signal for SimpleFollowTrendCloseLongSignal {
    type Wrapper = SignalWrapper;
    fn signal_category(&self) -> SignalCategory {
        SignalCategory::CloseLong
    }
    fn set_signal_column(&self, lf: &LazyFrame) -> Result<LazyFrame, GlowError> {
        let signal = self.signal_category();
        let signal_col = signal.get_column();
        let select_columns = vec![col("start_time"), col(signal_col)];
        let fast_ema_col_title = format!("{}_fast_ema", &self.anchor_symbol.name);
        let slow_ema_col_title = format!("{}_slow_ema", &self.anchor_symbol.name);
        let signal_lf = lf
            .clone()
            .with_column(
                when(
                    col(&fast_ema_col_title).gt(col(&slow_ema_col_title)).and(
                        col(&slow_ema_col_title)
                            .shift(1)
                            .gt_eq(col(&fast_ema_col_title).shift(1)),
                    ),
                )
                .then(lit(1))
                .otherwise(lit(0))
                .alias(signal_col),
            )
            .select(select_columns);
        Ok(signal_lf)
    }
    fn update_signal_column(&self, data: &DataFrame) -> Result<DataFrame, GlowError> {
        let mut new_lf = data.clone().lazy();
        new_lf = self.set_signal_column(&new_lf)?;
        let new_df = new_lf.collect()?;
        let mut result_df = data.clone();
        let signal = self.signal_category();
        let column = signal.get_column();
        let series = new_df.column(column)?;
        let _ = result_df.replace(&column, series.to_owned());

        Ok(result_df)
    }
    fn patch_symbols_pair(
        &self,
        updated_symbols_pair: SymbolsPair,
    ) -> Result<Self::Wrapper, GlowError> {
        if self.anchor_symbol == updated_symbols_pair.anchor {
            return Ok(self.clone().into());
        }
        let updated = Self {
            anchor_symbol: updated_symbols_pair.anchor,
        };
        Ok(updated.into())
    }
}

impl Signal for SimpleFollowTrendCloseShortSignal {
    type Wrapper = SignalWrapper;
    fn signal_category(&self) -> SignalCategory {
        SignalCategory::CloseShort
    }

    fn set_signal_column(&self, lf: &LazyFrame) -> Result<LazyFrame, GlowError> {
        let signal = self.signal_category();
        let signal_col = signal.get_column();
        let select_columns = vec![col("start_time"), col(signal_col)];
        let fast_ema_col_title = format!("{}_fast_ema", &self.anchor_symbol.name);
        let slow_ema_col_title = format!("{}_slow_ema", &self.anchor_symbol.name);
        let signal_lf = lf
            .clone()
            .with_column(
                when(
                    col(&fast_ema_col_title).gt(col(&slow_ema_col_title)).and(
                        col(&slow_ema_col_title)
                            .shift(1)
                            .gt_eq(col(&fast_ema_col_title).shift(1)),
                    ),
                )
                .then(lit(1))
                .otherwise(lit(0))
                .alias(signal_col),
            )
            .select(select_columns);
        Ok(signal_lf)
    }

    fn update_signal_column(&self, data: &DataFrame) -> Result<DataFrame, GlowError> {
        let mut new_lf = data.clone().lazy();
        new_lf = self.set_signal_column(&new_lf)?;
        let new_df = new_lf.collect()?;
        let mut result_df = data.clone();
        let signal = self.signal_category();
        let column = signal.get_column();
        let series = new_df.column(column)?;
        let _ = result_df.replace(&column, series.to_owned());

        Ok(result_df)
    }
    fn patch_symbols_pair(
        &self,
        updated_symbols_pair: SymbolsPair,
    ) -> Result<Self::Wrapper, GlowError> {
        if self.anchor_symbol == updated_symbols_pair.anchor {
            return Ok(self.clone().into());
        }
        let updated = Self {
            anchor_symbol: updated_symbols_pair.anchor,
        };
        Ok(updated.into())
    }
}

impl From<SimpleFollowTrendShortSignal> for SignalWrapper {
    fn from(value: SimpleFollowTrendShortSignal) -> Self {
        Self::SimpleFollowTrendShortSignal(value)
    }
}

impl From<SimpleFollowTrendLongSignal> for SignalWrapper {
    fn from(value: SimpleFollowTrendLongSignal) -> Self {
        Self::SimpleFollowTrendLongSignal(value)
    }
}

impl From<SimpleFollowTrendCloseShortSignal> for SignalWrapper {
    fn from(value: SimpleFollowTrendCloseShortSignal) -> Self {
        Self::SimpleFollowTrendCloseShortSignal(value)
    }
}

impl From<SimpleFollowTrendCloseLongSignal> for SignalWrapper {
    fn from(value: SimpleFollowTrendCloseLongSignal) -> Self {
        Self::SimpleFollowTrendCloseLongSignal(value)
    }
}
