use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(untagged)]
pub enum SerializedValue {
    StringValue(String),
    IntValue(i32),
    IntListValue(Vec<i32>),
}
