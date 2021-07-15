use serde::{de::DeserializeOwned, Deserialize, Serialize};
use std::fs;

static FILE: &str = "serialize.ron";

pub fn serialize<T: Serialize>(data: &T) {
    let config = ron::ser::PrettyConfig::new();
    let data = ron::ser::to_string_pretty(data, config).unwrap();
    fs::write(FILE, &data).unwrap();
}

pub fn deserialize<T: DeserializeOwned>() -> T {
    let data = fs::read_to_string(FILE).unwrap();
    let data = ron::de::from_str(&data).unwrap();

    data
}
