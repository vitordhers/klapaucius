use chrono::Duration as ChronoDuration;
use serde::{Deserialize, Serialize};
use std::time::Duration;

#[allow(non_camel_case_types)]
#[derive(Serialize, Deserialize, Debug, Default, Clone, Copy)]
pub enum Granularity {
    #[default]
    m1,
    m3,
    m5,
    m10,
    m15,
    m30,
    h1,
    h2,
    h4,
    h6,
    h12,
    d1,
    w1,
    M1,
}

impl Granularity {
    pub fn get_granularity_in_secs(&self) -> u32 {
        match self {
            Self::m1 => 60,
            Self::m3 => 3 * 60,
            Self::m5 => 5 * 60,
            Self::m10 => 10 * 60,
            Self::m15 => 15 * 60,
            Self::m30 => 30 * 60,
            Self::h1 => 60 * 60,
            Self::h2 => 2 * 60 * 20,
            Self::h4 => 4 * 60 * 60,
            Self::h6 => 6 * 60 * 60,
            Self::h12 => 12 * 60 * 60,
            Self::d1 => 1 * 24 * 60 * 60,
            Self::w1 => 7 * 24 * 60 * 60,
            Self::M1 => 30 * 24 * 60 * 60,
        }
    }

    pub fn get_granularity_in_mins(&self) -> u32 {
        let seconds = self.get_granularity_in_secs();
        seconds / 60
    }

    pub fn get_duration(&self) -> Duration {
        let seconds = self.get_granularity_in_secs();
        Duration::new(seconds.into(), 0)
    }

    pub fn get_chrono_duration(&self) -> ChronoDuration {
        let seconds = self.get_granularity_in_secs();
        ChronoDuration::seconds(seconds.into())
    }
}
