use common::{
    enums::signal_category::SignalCategory, structs::SymbolsPair, traits::signal::Signal,
};
use glow_error::GlowError;
use polars::prelude::*;
mod simple_follow_trend_signals;
pub use simple_follow_trend_signals::*;
#[derive(Clone, Debug)]
pub enum SignalWrapper {
    SimpleFollowTrendShortSignal(SimpleFollowTrendShortSignal),
    SimpleFollowTrendLongSignal(SimpleFollowTrendLongSignal),
    SimpleFollowTrendCloseShortSignal(SimpleFollowTrendCloseShortSignal),
    SimpleFollowTrendCloseLongSignal(SimpleFollowTrendCloseLongSignal),
}

impl Signal for SignalWrapper {
    type Wrapper = Self;
    fn signal_category(&self) -> SignalCategory {
        match self {
            Self::SimpleFollowTrendShortSignal(sig) => sig.signal_category(),
            Self::SimpleFollowTrendLongSignal(sig) => sig.signal_category(),
            Self::SimpleFollowTrendCloseShortSignal(sig) => sig.signal_category(),
            Self::SimpleFollowTrendCloseLongSignal(sig) => sig.signal_category(),
        }
    }
    fn set_signal_column(&self, lf: &LazyFrame) -> Result<LazyFrame, GlowError> {
        match self {
            Self::SimpleFollowTrendShortSignal(sig) => sig.set_signal_column(lf),
            Self::SimpleFollowTrendLongSignal(sig) => sig.set_signal_column(lf),
            Self::SimpleFollowTrendCloseShortSignal(sig) => sig.set_signal_column(lf),
            Self::SimpleFollowTrendCloseLongSignal(sig) => sig.set_signal_column(lf),
        }
    }
    fn update_signal_column(&self, data: &DataFrame) -> Result<DataFrame, GlowError> {
        match self {
            Self::SimpleFollowTrendShortSignal(sig) => sig.update_signal_column(data),
            Self::SimpleFollowTrendLongSignal(sig) => sig.update_signal_column(data),
            Self::SimpleFollowTrendCloseShortSignal(sig) => sig.update_signal_column(data),
            Self::SimpleFollowTrendCloseLongSignal(sig) => sig.update_signal_column(data),
        }
    }
    fn patch_symbols_pair(&self, symbols_pair: SymbolsPair) -> Result<Self::Wrapper, GlowError> {
        match self {
            Self::SimpleFollowTrendShortSignal(sig) => Ok(sig.patch_symbols_pair(symbols_pair)?),
            Self::SimpleFollowTrendLongSignal(sig) => Ok(sig.patch_symbols_pair(symbols_pair)?),
            Self::SimpleFollowTrendCloseShortSignal(sig) => {
                Ok(sig.patch_symbols_pair(symbols_pair)?)
            }
            Self::SimpleFollowTrendCloseLongSignal(sig) => {
                Ok(sig.patch_symbols_pair(symbols_pair)?)
            }
        }
    }
}
