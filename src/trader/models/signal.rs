use super::super::errors::Error;
use polars::prelude::*;

#[allow(dead_code)]
#[derive(PartialEq, Eq, Hash)]
pub enum SignalCategory {
    KeepPosition,
    GoLong,
    GoShort,
    RevertLong,
    RevertShort,
    RevertPosition,
    CloseLong,
    CloseShort,
    ClosePosition,
}

pub trait Signer {
    fn signal_category(&self) -> SignalCategory;
    fn compute_signal_column(&self, lf: &LazyFrame) -> Result<LazyFrame, Error>;
}
