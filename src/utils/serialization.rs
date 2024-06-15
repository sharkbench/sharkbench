use std::fmt::{Display, Formatter};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(untagged)]
pub enum SerializedValue {
    StringValue(String),
    IntValue(i32),
    IntListValue(Vec<i32>),
}

impl Display for SerializedValue {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            SerializedValue::StringValue(s) => write!(f, "{}", s),
            SerializedValue::IntValue(i) => write!(f, "{}", i),
            SerializedValue::IntListValue(list) => {
                write!(f, "[")?;
                let mut first = true;
                for item in list {
                    if !first {
                        write!(f, ", ")?;
                    }
                    write!(f, "{}", item)?;
                    first = false;
                }
                write!(f, "]")
            }
        }
    }
}
