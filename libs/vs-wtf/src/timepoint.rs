use std::{
    ops::{Add, Sub},
    time::SystemTime,
};

use crate::timespan::Timespan;

use super::Timecode;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub struct Timepoint {
    pub time: Timecode,
}
impl Default for Timepoint {
    fn default() -> Self {
        Timepoint {
            time: Timecode::zero(),
        }
    }
}

impl Timepoint {
    pub fn new(time: Timecode) -> Timepoint {
        Timepoint { time }
    }
    pub fn new_secs(secs: f64) -> Timepoint {
        Self::new(Timecode::new_secs(secs))
    }

    pub fn new_ms(ms: f64) -> Timepoint {
        Self::new(Timecode::new_ms(ms))
    }

    pub fn new_us(us: f64) -> Timepoint {
        Self::new(Timecode::new_us(us))
    }

    pub fn new_ns(ns: u128) -> Timepoint {
        Self::new(Timecode::new_ns(ns))
    }

    pub fn now() -> Timepoint {
        let now_nano = match SystemTime::now().duration_since(SystemTime::UNIX_EPOCH) {
            Ok(n) => n.as_nanos(),
            Err(_) => panic!("SystemTime before UNIX EPOCH!"),
        };

        Timepoint::new(Timecode::new_ns(now_nano))
    }

    pub fn zero() -> Timepoint {
        Timepoint {
            time: Timecode::zero(),
        }
    }

    pub fn secs(&self) -> f64 {
        self.time.secs()
    }

    pub fn ms(&self) -> f64 {
        self.time.ms()
    }

    pub fn us(&self) -> f64 {
        self.time.us()
    }

    pub fn ns(&self) -> u128 {
        self.time.ns()
    }
}

impl Add<Timespan> for Timepoint {
    type Output = Timepoint;

    fn add(self, rhs: Timespan) -> Self::Output {
        let duration_time = rhs.time;
        let new_time = self.time + duration_time;
        Timepoint::new(new_time)
    }
}

/// Subtract operation.
/// Timepoint - Timepoint = Timespan
impl Sub<Timepoint> for Timepoint {
    type Output = Timespan;

    fn sub(self, rhs: Timepoint) -> Self::Output {
        let duration_time = self.time - rhs.time;

        Timespan::new(duration_time)
    }
}

impl From<Timecode> for Timepoint {
    fn from(time: Timecode) -> Self {
        Timepoint { time }
    }
}

#[cfg(test)]
mod tests_constructors {
    use super::*;

    #[test]
    fn test_new() {
        let time = Timecode::new(1, 0);
        let timepoint = Timepoint::new(time);
        assert_eq!(timepoint.time, time);
    }

    #[test]
    fn test_new_conversions() {
        let time = Timecode::new(1, 0);
        let a = Timepoint::new(time);
        let b = Timepoint::new_secs(1.0);
        let c = Timepoint::new_ms(1000.0);
        let d = Timepoint::new_us(1_000_000.0);
        let e = Timepoint::new_ns(1_000_000_000);

        assert_eq!(a, b);
        assert_eq!(a, c);
        assert_eq!(a, d);
        assert_eq!(a, e);
    }

    #[test]
    fn test_now() {
        let now = Timepoint::now();
        let now_nano = match SystemTime::now().duration_since(SystemTime::UNIX_EPOCH) {
            Ok(n) => n.as_nanos(),
            Err(_) => panic!("SystemTime before UNIX EPOCH!"),
        };
        let now_time = Timecode::new_ns(now_nano);
        assert!(
            now.time.secs() - now_time.secs() < 1.0,
            "Timepoint::now() and SystemTime::now() are within 1 second of each other"
        );
    }

    #[test]
    fn test_zero() {
        let zero = Timepoint::zero();
        let zero_time = Timecode::default(); // Should be 0 seconds
        assert_eq!(zero.time, zero_time);
    }
}
#[cfg(test)]
mod tests_conversions {
    use super::*;

    #[test]
    fn test_conversions() {
        let time = Timecode::new(1, 500_000_000);
        let timepoint = Timepoint::new(time);
        assert_eq!(timepoint.secs(), 1.5);
        assert_eq!(timepoint.ms(), 1500.0);
        assert_eq!(timepoint.us(), 1_500_000.0);
        assert_eq!(timepoint.ns(), 1_500_000_000);
    }

    #[test]
    fn test_conversions_into_timecode() {
        let time = Timecode::new(1, 500_000_000);
        let timepoint: Timepoint = time.into();

        assert_eq!(time, timepoint.time);
    }
}

#[cfg(test)]
mod tests_math {
    use super::*;

    #[test]
    fn test_add() {
        let a = Timepoint::new_secs(1.0);
        let b = Timespan::new_secs(0.5);
        let c = a + b;
        assert_eq!(c.time.secs, 1);
        assert_eq!(c.time.nanos, 500_000_000);
    }

    #[test]
    fn test_sub() {
        let a = Timepoint::new_secs(1.0);
        let b = Timepoint::new_secs(0.5);
        let c = a - b;
        assert_eq!(c.time.secs, 0);
        assert_eq!(c.time.nanos, 500_000_000);
    }
}
