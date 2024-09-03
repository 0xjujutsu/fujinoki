use serde_json::Value as JsonValue;

pub fn merge_json(base: &mut JsonValue, other: &JsonValue) {
    match (base, other) {
        (JsonValue::Object(base_map), JsonValue::Object(other_map)) => {
            for (key, other_value) in other_map {
                match base_map.get_mut(key) {
                    Some(base_value) => merge_json(base_value, other_value),
                    None => {
                        base_map.insert(key.clone(), other_value.clone());
                    }
                }
            }
        }
        (base, other) => {
            *base = other.clone();
        }
    }
}
