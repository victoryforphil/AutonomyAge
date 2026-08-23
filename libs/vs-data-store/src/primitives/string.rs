use super::Primitives;

impl From<&str> for Primitives {
    fn from(value: &str) -> Self {
        Primitives::Text(value.to_string())
    }
}

impl From<String> for Primitives {
    fn from(value: String) -> Self {
        Primitives::Text(value)
    }
}

impl From<Primitives> for String {
    fn from(value: Primitives) -> Self {
        match value {
            Primitives::Text(v) => v,
            _ => panic!("Cannot convert {:?} to String", value),
        }
    }
}
