use crate::internal::*;

pub type Map = std::collections::BTreeMap<Value, Value>;
pub type Set = std::collections::BTreeSet<Value>;
pub type Float = ordered_float::OrderedFloat<f64>;

#[derive(Debug, Clone, PartialEq, PartialOrd, Eq, Ord)]
pub enum Value {
    Null,
    Bool(bool),
    Int(i64),
    Float(Float),
    BigNumber(String),
    Error(String),
    String(String),
    Bytes(Vec<u8>),
    Array(Vec<Value>),
    Map(Map),
    Set(Set),
    Attribute(Map, Box<Value>),
    Push(String, Vec<Value>),
    Done
}

impl Value {
    pub fn new(x: impl Into<Value>) -> Value {
        x.into()
    }

    pub fn done() -> Result<Value, Error> {
        Ok(Value::Done)
    }

    pub fn error(x: impl Into<String>) -> Value {
        Value::Error(x.into())
    }

    pub fn ok() -> Value {
        Value::String("OK".into())
    }

    pub async fn write<W: Send + Unpin + AsyncWrite>(&self, w: &mut W) -> Result<(), Error> {
        let mut enc = Encoder::new(w);
        enc.encode(self).await?;
        enc.flush().await
    }

    pub async fn read<R: Send + Unpin + AsyncRead>(r: &mut R) -> Result<Value, Error> {
        let mut d = Decoder::new(r);
        d.decode().await
    }

    pub fn is_null(&self) -> bool {
        self == &Value::Null
    }

    pub fn as_bool(&self) -> Option<bool> {
        if let Value::Bool(b) = self {
            return Some(*b);
        }

        None
    }

    pub fn as_bool_mut(&mut self) -> Option<&mut bool> {
        if let Value::Bool(b) = self {
            return Some(b);
        }

        None
    }

    pub fn as_int(&self) -> Option<i64> {
        match self {
            Value::Int(x) => Some(*x),
            Value::Float(f) => Some(f.into_inner() as i64),
            Value::String(s) => match s.parse() {
                Ok(x) => Some(x),
                Err(_) => None,
            },
            _ => None,
        }
    }

    pub fn as_int_mut(&mut self) -> Option<&mut i64> {
        if let Value::Int(x) = self {
            return Some(x);
        }

        None
    }

    pub fn as_float(&self) -> Option<f64> {
        if let Value::Float(x) = self {
            return Some((*x).into());
        }

        match self {
            Value::Float(x) => Some(x.clone().into()),
            Value::Int(x) => Some(*x as f64),
            Value::String(s) => match s.parse() {
                Ok(x) => Some(x),
                Err(_) => None,
            },
            _ => None,
        }
    }

    pub fn as_float_mut(&mut self) -> Option<&mut f64> {
        if let Value::Float(x) = self {
            return Some(x.as_mut());
        }

        None
    }

    pub fn as_error(&self) -> Option<&str> {
        if let Value::Error(e) = self {
            return Some(e.as_str());
        }

        None
    }

    pub fn as_error_mut(&mut self) -> Option<&mut String> {
        if let Value::Error(e) = self {
            return Some(e);
        }

        None
    }

    pub fn as_string(&self) -> Option<&str> {
        if let Value::String(s) = self {
            return Some(s.as_str());
        }

        None
    }

    pub fn as_string_mut(&mut self) -> Option<&mut String> {
        if let Value::String(s) = self {
            return Some(s);
        }

        None
    }

    pub fn as_bytes(&self) -> Option<&[u8]> {
        match self {
            Value::Bytes(x) => Some(x.as_slice()),
            Value::String(s) => Some(s.as_bytes()),
            _ => None,
        }
    }

    pub fn as_bytes_mut(&mut self) -> Option<&mut Vec<u8>> {
        if let Value::Bytes(x) = self {
            return Some(x);
        }

        None
    }

    pub fn as_array(&self) -> Option<&[Value]> {
        if let Value::Array(a) = self {
            return Some(a.as_slice());
        }

        None
    }

    pub fn as_array_mut(&mut self) -> Option<&mut Vec<Value>> {
        if let Value::Array(a) = self {
            return Some(a);
        }

        None
    }

    pub fn as_map(&self) -> Option<&Map> {
        if let Value::Map(m) = self {
            return Some(m);
        }

        None
    }

    pub fn as_map_mut(&mut self) -> Option<&mut Map> {
        if let Value::Map(m) = self {
            return Some(m);
        }

        None
    }

    pub fn as_set(&self) -> Option<&Set> {
        if let Value::Set(s) = self {
            return Some(s);
        }

        None
    }

    pub fn as_set_mut(&mut self) -> Option<&mut Set> {
        if let Value::Set(s) = self {
            return Some(s);
        }

        None
    }
}

impl From<()> for Value {
    fn from(_: ()) -> Value {
        Value::Null
    }
}

impl<T: Into<Value>> From<Option<T>> for Value {
    fn from(x: Option<T>) -> Value {
        match x {
            Some(x) => x.into(),
            None => Value::Null,
        }
    }
}

impl std::convert::TryFrom<Value> for () {
    type Error = Error;

    fn try_from(value: Value) -> Result<Self, Self::Error> {
        if let Value::Null = value {
            return Ok(());
        }

        Err(Error::InvalidValue(value))
    }
}

impl From<bool> for Value {
    fn from(b: bool) -> Value {
        Value::Bool(b)
    }
}

impl std::convert::TryFrom<Value> for bool {
    type Error = Error;

    fn try_from(value: Value) -> Result<Self, Self::Error> {
        if let Value::Bool(b) = value {
            return Ok(b);
        }

        Err(Error::InvalidValue(value))
    }
}

impl From<i64> for Value {
    fn from(x: i64) -> Value {
        Value::Int(x)
    }
}

impl std::convert::TryFrom<Value> for i64 {
    type Error = Error;

    fn try_from(value: Value) -> Result<Self, Self::Error> {
        if let Value::Int(b) = value {
            return Ok(b);
        }

        Err(Error::InvalidValue(value))
    }
}

impl From<f64> for Value {
    fn from(x: f64) -> Value {
        Value::Float(x.into())
    }
}

impl std::convert::TryFrom<Value> for f64 {
    type Error = Error;

    fn try_from(value: Value) -> Result<Self, Self::Error> {
        if let Value::Float(b) = value {
            return Ok(b.into());
        }

        Err(Error::InvalidValue(value))
    }
}

impl<E: std::fmt::Debug> From<Result<Value, E>> for Value {
    fn from(x: Result<Value, E>) -> Value {
        match x {
            Ok(x) => x,
            Err(e) => Value::Error(format!("{:?}", e)),
        }
    }
}

impl From<String> for Value {
    fn from(x: String) -> Value {
        Value::String(x)
    }
}

impl std::convert::TryFrom<Value> for String {
    type Error = Error;

    fn try_from(value: Value) -> Result<Self, Self::Error> {
        if let Value::String(b) = value {
            return Ok(b);
        }

        Err(Error::InvalidValue(value))
    }
}

impl<'a> From<&'a str> for Value {
    fn from(x: &'a str) -> Value {
        Value::String(x.into())
    }
}

impl From<Vec<u8>> for Value {
    fn from(x: Vec<u8>) -> Value {
        Value::Bytes(x)
    }
}

impl std::convert::TryFrom<Value> for Vec<u8> {
    type Error = Error;

    fn try_from(value: Value) -> Result<Self, Self::Error> {
        if let Value::Bytes(b) = value {
            return Ok(b);
        }

        Err(Error::InvalidValue(value))
    }
}

impl<'a> From<&'a [u8]> for Value {
    fn from(x: &'a [u8]) -> Value {
        Value::Bytes(x.into())
    }
}

impl From<Vec<Value>> for Value {
    fn from(x: Vec<Value>) -> Value {
        Value::Array(x)
    }
}

impl std::convert::TryFrom<Value> for Vec<Value> {
    type Error = Error;

    fn try_from(value: Value) -> Result<Self, Self::Error> {
        if let Value::Array(b) = value {
            return Ok(b);
        }

        Err(Error::InvalidValue(value))
    }
}

impl From<Map> for Value {
    fn from(x: Map) -> Value {
        Value::Map(x)
    }
}

impl std::convert::TryFrom<Value> for Map {
    type Error = Error;

    fn try_from(value: Value) -> Result<Self, Self::Error> {
        if let Value::Map(b) = value {
            return Ok(b);
        }

        Err(Error::InvalidValue(value))
    }
}

impl From<Set> for Value {
    fn from(x: Set) -> Value {
        Value::Set(x)
    }
}

impl std::convert::TryFrom<Value> for Set {
    type Error = Error;

    fn try_from(value: Value) -> Result<Self, Self::Error> {
        if let Value::Set(b) = value {
            return Ok(b);
        }

        Err(Error::InvalidValue(value))
    }
}

#[macro_export]
macro_rules! array {
    [$($x:expr),*$(,)?] => {
        $crate::Value::Array(vec![$($x.into()),*])
    }
}

#[macro_export]
macro_rules! map {
    {$($k:expr => $v:expr),*$(,)?} => {{
        #[allow(unused_mut)]
        let mut map = Map::new();
        $(
            map.insert($k.into(), $v.into());
        )*
        $crate::Value::Map(map)
    }}
}

#[macro_export]
macro_rules! set {
    {$($x:expr),*$(,)?} => {{
        #[allow(unused_mut)]
        let mut set = Set::new();
        $(
            set.insert($x.into());
        )*
        $crate::Value::Set(set)
    }}
}
