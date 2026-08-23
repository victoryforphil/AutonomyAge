use super::Primitives;

impl From<f64> for Primitives {
    fn from(value: f64) -> Self {
        Primitives::Float(value)
    }
}

impl From<Primitives> for f64 {
    fn from(value: Primitives) -> Self {
        match value {
            Primitives::Float(v) => v,
            _ => panic!("Cannot convert {:?} to f64", value),
        }
    }
}

impl From<f32> for Primitives {
    fn from(value: f32) -> Self {
        Primitives::Float(value as f64)
    }
}

impl From<Primitives> for f32 {
    fn from(value: Primitives) -> Self {
        match value {
            Primitives::Float(v) => v as f32,
            _ => panic!("Cannot convert {:?} to f32", value),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_float() {
        let value = 1.0;
        let primitive = Primitives::from(value);
        assert_eq!(primitive, Primitives::Float(1.0));
        let value = Primitives::Float(1.0);
        let float_value: f64 = value.into();
        assert_eq!(float_value, 1.0);
        let value = 1.0;
        let primitive = Primitives::from(value);
        assert_eq!(primitive, Primitives::Float(1.0));
        let value = Primitives::Float(1.0);
        let float_value: f32 = value.into();
        assert_eq!(float_value, 1.0);
    }
}
