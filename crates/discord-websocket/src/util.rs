use serde_json::{Map, Value as JsonValue};

pub trait CamelCaseJson {
    fn camel_case_json(&self) -> Self;
}

impl CamelCaseJson for Map<String, JsonValue> {
    fn camel_case_json(&self) -> Self {
        let mut new_json = Map::new();
        for (key, value) in self.iter() {
            let new_key = key
                .chars()
                .enumerate()
                .fold(String::new(), |mut acc, (i, c)| {
                    if i == 0 {
                        acc.push(c.to_ascii_lowercase());
                    } else if c.is_uppercase() {
                        acc.push('_');
                        acc.push(c.to_ascii_lowercase());
                    } else {
                        acc.push(c);
                    }
                    acc
                });
            new_json.insert(new_key, value.clone());
        }
        new_json
    }
}
