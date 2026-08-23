use serde::{Deserialize, Serialize};
use std::fmt;
use vs_wtf::Timespan;

#[derive(Clone, Default, Serialize, Deserialize)]
pub struct RetentionPolicy {
    pub max_age: Option<Timespan>,
    pub max_rows: Option<usize>,
}

impl fmt::Debug for RetentionPolicy {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "[max_age = {:?}, max_rows = {:?}]",
            self.max_age, self.max_rows
        )
    }
}
