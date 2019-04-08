use serde::{Deserialize, Deserializer};
use serde_derive::Deserialize;
use std::collections::HashMap;

//#[extract("...")]
#[macro_export]
macro_rules! extract {
    ($name:ident, $attr:expr) => {
        paste::item! {
            pub fn [<extract_ $name>]<'de, T, D>(deserializer: D) -> Result<T, D::Error>
                where
                    T: serde::Deserialize<'de> + Default + std::fmt::Debug + Clone,
                    D: serde::Deserializer<'de>,
            {
                crate::de::extract_attr(deserializer, $attr)
            }
        }
    };
}

extract!(text, "#text");
extract!(value, "$value");

#[macro_export]
macro_rules! deflate {
($name:ident, $attr:expr) => {
    paste::item! {
            pub fn [<deflate_ $name>]<'de, D>(deserializer: D) -> Result<HashMap<String, serde_json::Value>, D::Error>
                where
                    D: Deserializer<'de>,
            {
                deflate_attr(deserializer, $attr)
            }
        }
    }
}

deflate!(text, "#text");
deflate!(value, "$value");

//#[into(seq)]
pub fn into_seq<'de, T, D>(deserializer: D) -> Result<Vec<T>, D::Error>
where
    T: Deserialize<'de>,
    D: Deserializer<'de>,
{
    #[derive(Debug, Deserialize)]
    #[serde(untagged)]
    enum Value<T> {
        Into(Vec<T>),
        From(T),
    }

    let result: Result<Value<T>, D::Error> = Deserialize::deserialize(deserializer);
    if let Ok(value) = result {
        match value {
            Value::Into(v) => Ok(v),
            Value::From(v) => Ok(vec![v]),
        }
    } else {
        Ok(vec![])
    }
}

//#[into(str)]
pub fn into_str<'de, D>(deserializer: D) -> Result<String, D::Error>
where
    D: Deserializer<'de>,
{
    maybe_into_str(deserializer).map(|v| match v {
        Some(s) => s,
        None => String::new(),
    })
}

//#[from(str)]
pub fn from_str<'de, T, D>(deserializer: D) -> Result<T, D::Error>
where
    T: Deserialize<'de> + std::str::FromStr + Default,
    D: Deserializer<'de>,
    <T as std::str::FromStr>::Err: std::fmt::Display,
{
    use serde::de;

    #[derive(Debug, Deserialize)]
    #[serde(untagged)]
    enum Value<T> {
        From(String),
        Into(T),
    }

    let result: Result<Value<T>, D::Error> = Deserialize::deserialize(deserializer);
    if let Ok(s) = result {
        match s {
            Value::From(v) => v.parse::<T>().map_err(de::Error::custom),
            Value::Into(v) => Ok(v),
        }
    } else {
        Ok(T::default())
    }
}

//#[maybe(str)]
pub fn maybe_str<'de, D>(deserializer: D) -> Result<Option<String>, D::Error>
where
    D: Deserializer<'de>,
{
    let result: Result<String, D::Error> = Deserialize::deserialize(deserializer);
    if let Ok(value) = result {
        if value.trim().is_empty() {
            Ok(None)
        } else {
            Ok(Some(value))
        }
    } else {
        Ok(None)
    }
}

//#[maybe_into(str)]
fn maybe_into_str<'de, D>(deserializer: D) -> Result<Option<String>, D::Error>
where
    D: Deserializer<'de>,
{
    #[derive(Debug, Deserialize)]
    #[serde(untagged)]
    enum Value {
        String(String),
        Int(i64),
        Uint(u64),
        Float(f64),
        Bool(bool),
        Char(char),
    }

    let result: Result<Value, D::Error> = Deserialize::deserialize(deserializer);
    if let Ok(value) = result {
        match value {
            Value::String(v) => {
                if v.trim().is_empty() {
                    Ok(None)
                } else {
                    Ok(Some(v))
                }
            }
            Value::Int(v) => Ok(Some(v.to_string())),
            Value::Uint(v) => Ok(Some(v.to_string())),
            Value::Float(v) => Ok(Some(v.to_string())),
            Value::Bool(v) => Ok(Some(v.to_string())),
            Value::Char(v) => Ok(Some(v.to_string())),
        }
    } else {
        Ok(None)
    }
}

fn extract_attr<'de, T, D>(deserializer: D, attr: &str) -> Result<T, D::Error>
where
    T: Deserialize<'de> + Default + std::fmt::Debug + Clone,
    D: Deserializer<'de>,
{
    use std::collections::HashMap;

    #[derive(Debug, Deserialize)]
    #[serde(untagged)]
    enum Value<T> {
        From(HashMap<String, T>),
        Into(T),
    }

    let result: Result<Value<T>, D::Error> = serde::Deserialize::deserialize(deserializer);
    if let Ok(value) = result {
        match value {
            Value::From(map) => {
                if let Some(value) = map.get(attr) {
                    Ok(value.clone())
                } else {
                    Ok(T::default())
                }
            }
            Value::Into(v) => Ok(v),
        }
    } else {
        Ok(T::default())
    }
}

fn deflate_attr<'de, D>(
    deserializer: D,
    attr: &str,
) -> Result<HashMap<String, serde_json::Value>, D::Error>
where
    D: Deserializer<'de>,
{
    let mut de_map = HashMap::new();

    let result: Result<HashMap<String, serde_json::Value>, D::Error> =
        Deserialize::deserialize(deserializer);
    if let Ok(map) = result {
        for entry in map.iter() {
            deflate_attr_in(entry, attr, &mut de_map);
        }
    }

    Ok(de_map)
}

fn deflate_attr_in(
    entry: (&String, &serde_json::Value),
    attr: &str,
    de_map: &mut HashMap<String, serde_json::Value>,
) {
    use serde_json::Value::Object;

    let (key, value) = entry;

    if let Object(map) = value {
        if let Some(value) = map.get(attr) {
            de_map.insert(key.to_owned(), value.clone());
        } else {
            for entry in map {
                deflate_attr_in(entry, attr, de_map);
            }
        }
    } else {
        de_map.insert(key.to_owned(), value.clone());
    }
}
