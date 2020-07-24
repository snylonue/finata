use lazy_static::lazy_static;

#[macro_export]
macro_rules! value_to_string {
    ($v: expr) => {
        match $v {
            serde_json::Value::String(ref s) => Some(s.to_owned()),
            _ => None,
        }
    };
    ($v: expr, $or: expr) => {
        match $v {
            serde_json::Value::String(ref s) => Some(s.to_owned()),
            _ => $crate::value_to_string!($or),
        }
    };
}
#[macro_export]
macro_rules! hdmap {
    () => {
        reqwest::header::HeaderMap::new()
    };
    ($($key: expr => $value: expr),+ $(,)?) => {
        {
            use std::convert::TryInto;
            const CAP: usize = sugars::count!($($key),*);
            let mut map = reqwest::header::HeaderMap::with_capacity(CAP);
            $(map.insert($key, $value.try_into().unwrap());)+
            map
        }
    };
}

lazy_static! {
    pub static ref CLIENT: reqwest::Client = reqwest::Client::new();
    pub static ref UA: reqwest::header::HeaderValue = reqwest::header::HeaderValue::from_static("Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/75.0.3770.142 Safari/537.36");
}