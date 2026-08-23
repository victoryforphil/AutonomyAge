use std::ops::{Add, Sub};

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
pub struct Timecode {
    pub(crate) secs: u64,
    pub(crate) nanos: u32,
}

impl PartialOrd for Timecode {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for Timecode {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.ns().cmp(&other.ns())
    }
}

impl Timecode {
    pub fn new_secs(secs: f64) -> Timecode {
        let secs = secs;
        let nanos = (secs.fract() * 1_000_000_000.0) as u32;
        Timecode {
            secs: secs as u64,
            nanos,
        }
    }

    pub fn new(secs: u64, nanos: u32) -> Timecode {
        Timecode { secs, nanos }
    }

    pub fn new_ns(nanos: u128) -> Timecode {
        let secs = (nanos / 1_000_000_000) as u64;
        let nanos = (nanos % 1_000_000_000) as u32;
        Timecode { secs, nanos }
    }

    pub fn zero() -> Timecode {
        Timecode { secs: 0, nanos: 0 }
    }

    pub fn new_hz(hz: f64) -> Timecode {
        let secs = 1.0 / hz;
        Timecode::new_secs(secs)
    }

    pub fn new_ms(ms: f64) -> Timecode {
        let secs = ms / 1000.0;
        Timecode::new_secs(secs)
    }

    pub fn new_us(us: f64) -> Timecode {
        let secs = us / 1_000_000.0;
        Timecode::new_secs(secs)
    }

    pub fn secs(&self) -> f64 {
        self.secs as f64 + (self.nanos as f64 / 1_000_000_000.0)
    }

    pub fn ms(&self) -> f64 {
        self.secs as f64 * 1000.0 + (self.nanos as f64 / 1_000_000.0)
    }

    pub fn us(&self) -> f64 {
        self.secs as f64 * 1_000_000.0 + (self.nanos as f64 / 1_000.0)
    }
    pub fn hz(&self) -> f64 {
        1.0 / self.secs()
    }

    pub fn ns(&self) -> u128 {
        (self.secs as u128 * 1_000_000_000) + (self.nanos as u128)
    }
}

impl Sub for Timecode {
    type Output = Timecode;

    fn sub(self, rhs: Timecode) -> Timecode {
        let secs = self.secs() - rhs.secs();
        Timecode::new_secs(secs)
    }
}

impl Add for Timecode {
    type Output = Timecode;

    fn add(self, rhs: Timecode) -> Timecode {
        let secs = self.secs() + rhs.secs();
        Timecode::new_secs(secs)
    }
}

#[cfg(test)]
mod tests_constructors {
    use super::*;

    #[test]
    fn test_timecode_default() {
        let a = Timecode::default();
        assert_eq!(a.secs, 0);
        assert_eq!(a.nanos, 0);
    }

    #[test]
    fn test_timecode_new() {
        let a = Timecode::new_secs(2.0);
        let b = Timecode::new(2, 0);
        let c = Timecode::new_ns(2_000_000_000);
        let d = Timecode::new_us(2_000_000.0);
        let e = Timecode::new_ms(2000.0);
        let f = Timecode::new_hz(0.5);

        assert_eq!(a, b, "1.5s == 1s + 500_000_000ns");
        assert_eq!(a, c, "1.5s == 1_500_000_000ns");
        assert_eq!(a, d, "1.5s == 1_500_000us");
        assert_eq!(a, e, "1.5s == 1500ms");
        assert_eq!(a, f, "1.5s == 1/1.5Hz");
    }

    #[test]
    fn test_timecode_zero() {
        let a = Timecode::zero();
        assert_eq!(a.secs, 0);
        assert_eq!(a.nanos, 0);
    }
}

#[cfg(test)]
mod tests_conversions {
    use super::*;

    #[test]
    fn test_timecode_secs() {
        let a = Timecode::new_secs(1.5);
        assert_eq!(a.secs(), 1.5);
    }

    #[test]
    fn test_timecode_ms() {
        let a = Timecode::new_secs(1.5);
        assert_eq!(a.ms(), 1500.0);
    }

    #[test]
    fn test_timecode_us() {
        let a = Timecode::new_secs(1.5);
        assert_eq!(a.us(), 1_500_000.0);
    }

    #[test]
    fn test_timecode_hz() {
        let a = Timecode::new_secs(1.0);
        assert_eq!(a.hz(), 1.0);
    }

    #[test]
    fn test_timecode_ns() {
        let a = Timecode::new_secs(1.5);
        assert_eq!(a.ns(), 1_500_000_000);
    }
}

#[cfg(test)]
mod tests_math {
    use super::*;

    #[test]
    fn test_timecode_sub() {
        let a = Timecode::new_secs(1.0);
        let b = Timecode::new_secs(0.5);
        let c = a - b;
        assert_eq!(c.secs, 0);
        assert_eq!(c.nanos, 500_000_000);
    }

    #[test]
    fn test_timecode_sub_overflow() {
        let a = Timecode::new_secs(1.0);
        let b = Timecode::new_secs(1.0);
        let c = a - b;
        assert_eq!(c.secs, 0);
        assert_eq!(c.nanos, 0);
    }

    #[test]
    fn test_timecode_add() {
        let a = Timecode::new_secs(1.0);
        let b = Timecode::new_secs(0.5);
        let c = a + b;
        assert_eq!(c.secs, 1);
        assert_eq!(c.nanos, 500_000_000);
    }
}
