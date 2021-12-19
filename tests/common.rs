use serde_json::Value;

pub fn get_json_path<'a>(value: &'a Value, path: &str) -> &'a Value {
    path.split('.').into_iter().fold(value, |acc, fragment| {
        acc.get(fragment).unwrap_or(&serde_json::value::Value::Null)
    })
}

pub fn value_as_string(value: Option<&'_ Value>) -> String {
    value.unwrap_or(&Value::Null).to_string()
}
