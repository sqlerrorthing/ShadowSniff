use alloc::borrow::Cow;
use alloc::rc::Rc;
use alloc::string::String;
use alloc::sync::Arc;
use alloc::vec::Vec;

#[derive(Clone)]
pub enum Value<'p> {
    Null,
    String(Cow<'p, str>),
    Blob(Cow<'p, [u8]>),
    Int(i64),
    Float(f64),
}

impl Value<'_> {
    pub fn as_str(&self) -> Option<&str> {
        if let Value::String(s) = self {
            Some(s.as_ref())
        } else {
            None
        }
    }

    pub fn as_int(&self) -> Option<i64> {
        if let Value::Int(i) = self {
            Some(*i)
        } else {
            None
        }
    }
}

#[derive(Clone)]
pub enum OwnedValue {
    Null,
    String(Rc<String>),
    Blob(Rc<Vec<u8>>),
    Int(i64),
    Float(f64),
}

impl From<&OwnedValue> for crate::Value {
    fn from(value: &OwnedValue) -> Self {
        match value {
            OwnedValue::Null => crate::Value::Null,
            OwnedValue::Int(i) => crate::Value::Integer(*i),
            OwnedValue::Float(f) => crate::Value::Float(*f),
            OwnedValue::String(rc_str) => {
                crate::Value::String(Arc::<str>::from(rc_str.as_str()))
            }
            OwnedValue::Blob(rc_blob) => {
                crate::Value::Blob(Arc::<[u8]>::from(rc_blob.as_slice()))
            }
        }
    }
}

impl<'p> From<Value<'p>> for OwnedValue {
    fn from(value: Value<'p>) -> Self {
        match value {
            Value::Null => Self::Null,
            Value::Int(i) => Self::Int(i),
            Value::Float(f) => Self::Float(f),
            Value::Blob(b) => Self::Blob(Rc::new(b.into_owned())),
            Value::String(s) => Self::String(Rc::new(s.into_owned())),
        }
    }
}