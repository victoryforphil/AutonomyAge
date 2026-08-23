use crate::Timepoint;

use super::Timecode;
use serde::{Deserialize, Serialize};
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub struct Timespan {
    pub time: Timecode,
}

impl Timespan {
    pub fn new(time_duration: Timecode) -> Timespan {
        Timespan {
            time: time_duration,
        }
    }

    pub fn new_points(start: Timepoint, end: Timepoint) -> Timespan {
        end - start
    }

    pub fn new_secs(secs: f64) -> Timespan {
        Self::new(Timecode::new_secs(secs))
    }
    pub fn new_hz(hz: f64) -> Timespan {
        Self::new(Timecode::new_hz(hz))
    }

    pub fn new_ms(ms: f64) -> Timespan {
        Self::new(Timecode::new_ms(ms))
    }

    pub fn new_us(us: f64) -> Timespan {
        Self::new(Timecode::new_us(us))
    }

    pub fn new_ns(ns: u128) -> Timespan {
        Self::new(Timecode::new_ns(ns))
    }

    pub fn from_duration(duration: std::time::Duration) -> Timespan {
        let secs = duration.as_secs();
        let nanos = duration.subsec_nanos();
        Timespan {
            time: Timecode::new(secs, nanos),
        }
    }

    pub fn as_duration(&self) -> std::time::Duration {
        std::time::Duration::new(self.time.secs, self.time.nanos)
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

    pub fn hz(&self) -> f64 {
        self.time.hz()
    }

    pub fn zero() -> Timespan {
        Timespan {
            time: Timecode::zero(),
        }
    }
}

impl From<std::time::Duration> for Timespan {
    fn from(duration: std::time::Duration) -> Self {
        Timespan::from_duration(duration)
    }
}

#[cfg(test)]
mod tests_constructors {
    use super::*;
    #[test]
    fn test_new_timespan() {
        let time = Timecode::new_secs(1.0);
        let timespan = Timespan::new(time);
        assert_eq!(timespan.time.secs, 1);
        assert_eq!(timespan.time.nanos, 0);
    }

    #[test]
    fn test_new_points() {
        let start = Timepoint::new(Timecode::new_secs(1.0));
        let end = Timepoint::new(Timecode::new_secs(2.0));
        let timespan = Timespan::new_points(start, end);
        assert_eq!(timespan.time.secs, 1);
        assert_eq!(timespan.time.nanos, 0);
    }

    #[test]
    fn test_new_conversions() {
        let a = Timespan::new_secs(1.0);
        let b = Timespan::new_ms(1000.0);
        let c = Timespan::new_us(1_000_000.0);
        let d = Timespan::new_ns(1_000_000_000);
        let e = Timespan::new_hz(1.0);

        assert_eq!(a, b);
        assert_eq!(b, c);
        assert_eq!(c, d);
        assert_eq!(d, e);
    }

    #[test]
    fn test_new_zero() {
        let a = Timespan::zero();
        assert_eq!(a.time.secs, 0);
        assert_eq!(a.time.nanos, 0);
    }
}
#[cfg(test)]
mod tests_duration {
    use super::*;
    #[test]
    fn test_as_duration() {
        let a = Timespan::new_secs(1.0);
        let b = a.as_duration();
        assert_eq!(b.as_secs(), 1);
        assert_eq!(b.subsec_nanos(), 0);
    }

    #[test]
    fn test_from_duration() {
        let a = std::time::Duration::new(1, 0);
        let b = Timespan::from_duration(a);
        assert_eq!(b.time.secs, 1);
        assert_eq!(b.time.nanos, 0);
    }

    #[test]
    fn test_into_duration() {
        let a = std::time::Duration::new(1, 0);
        let b: Timespan = a.into();
        assert_eq!(b.time.secs, 1);
        assert_eq!(b.time.nanos, 0);
    }
}

#[cfg(test)]
mod tests_conversions {
    use super::*;

    #[test]
    fn test_timespan_conversions() {
        let a = Timespan::new_secs(1.0);
        assert_eq!(a.secs(), 1.0);
        assert_eq!(a.ms(), 1000.0);
        assert_eq!(a.us(), 1_000_000.0);
        assert_eq!(a.ns(), 1_000_000_000);
        assert_eq!(a.hz(), 1.0);
    }
}
