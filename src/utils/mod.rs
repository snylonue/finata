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

pub mod client;

pub use self::client::{Client, UA};
