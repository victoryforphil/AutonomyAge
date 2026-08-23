use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Deserialize, Serialize, Eq)]
pub struct VicBlob {
    pub data: Vec<u8>,
    pub length: u64,
    pub data_type: String,
    pub hash: String,
}

impl VicBlob {
    pub fn new(data: Vec<u8>, length: u64, data_type: String, hash: String) -> VicBlob {
        VicBlob {
            data,
            length,
            data_type,
            hash,
        }
    }

    pub fn new_from_data(data: Vec<u8>) -> VicBlob {
        let length = data.len() as u64;
        let data_type = String::from("raw_bytes");
        let hash = String::from("not_implemented");
        VicBlob {
            data,
            length,
            data_type,
            hash,
        }
    }
    pub fn new_empty() -> VicBlob {
        VicBlob {
            data: Vec::new(),
            length: 0,
            data_type: String::from(""),
            hash: String::from(""),
        }
    }
}

impl Default for VicBlob {
    fn default() -> VicBlob {
        VicBlob::new_empty()
    }
}
