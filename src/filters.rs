use std::collections::HashMap;

use tera::{to_value, try_get_value, Result as TeraResult, Value};

pub fn icon_filter(value: &Value, _: &HashMap<String, Value>) -> TeraResult<Value> {
    let vehicle = try_get_value!("icon", "value", String, value);
    let icon = match vehicle.to_lowercase().as_str() {
        "saloon" | "sedan" => "🚗",
        "suv" => "🚙",
        "lorry" => "🚚",
        _ => "🚘",
    };

    to_value(icon).map_err(Into::into)
}
