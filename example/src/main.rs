use serde_conv_derive::convert;
use serde_derive::Deserialize;

#[convert]
#[derive(Default, Deserialize, Debug)]
struct Pancakes {
    #[serde(rename = "a")]
    #[from(str)]
    pub foo: u32,
    #[serde(rename = "b")]
    #[into(str)]
    pub bar: String,
    #[extract = "#text"]
    pub id: u8,
}

fn main() {
    let p: serde_json::Result<Pancakes> = serde_json::from_str("{\"a\": \"42\", \"b\": 42, \"id\": {\"#text\": 42 }}");
    dbg!(p);
}
