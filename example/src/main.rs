use serde_conv_derive::convert;
use serde_derive::Deserialize;
use serde_json::Value;
use std::collections::HashMap;

#[convert]
#[derive(Deserialize, Debug)]
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

#[convert]
#[derive(Debug, Deserialize)]
#[serde(rename_all = "PascalCase")]
struct Street {
    #[serde(rename = "Postleitzahl")]
    plz: String,
    #[serde(flatten)]
    #[deflate = "$value"]
    values: HashMap<String, Value>,
}

fn main() {
    let p: Pancakes =
        serde_json::from_str("{\"a\": \"42\", \"b\": 42, \"id\": {\"#text\": 42 }}").unwrap();
    dbg!(p);
    let xml = r#"
        <Street>
            <Postleitzahl>22453</Postleitzahl>
            <Person>
                <Name>Alfred</Name>
                <Age>042</Age>
            </Person>
            <Foo>
                <Bar>Test</Bar>
            </Foo>
            <Test>Foo</Test>
        </Street>
    "#;
    let s: Street = serde_xml_rs::from_str(xml).unwrap();
    dbg!(s);
}
