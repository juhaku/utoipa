#![cfg(feature = "serde_json")]
use serde_json::Value;

#[deprecated(
    since = "2.0.0",
    note = "Favor serde native `.pointer(...)` function over custom json path function"
)]
pub fn get_json_path<'a>(value: &'a Value, path: &str) -> &'a Value {
    path.split('.').into_iter().fold(value, |acc, fragment| {
        let value = if fragment.starts_with('[') && fragment.ends_with(']') {
            let index = fragment
                .replace('[', "")
                .replace(']', "")
                .parse::<usize>()
                .unwrap();
            acc.get(index)
        } else {
            acc.get(fragment)
        };
        value.unwrap_or(&serde_json::value::Value::Null)
    })
}

pub fn value_as_string(value: Option<&'_ Value>) -> String {
    value.unwrap_or(&Value::Null).to_string()
}

#[allow(unused)]
pub fn assert_json_array_len(value: &Value, len: usize) {
    match value {
        Value::Array(array) => assert_eq!(
            len,
            array.len(),
            "wrong amount of parameters {} != {}",
            len,
            array.len()
        ),
        _ => unreachable!(),
    }
}

#[macro_export]
macro_rules! assert_value {
    ($value:expr=> $( $path:literal = $expected:literal, $error:literal)* ) => {{
        $(
            let actual = crate::common::value_as_string(Some(crate::common::get_json_path(&$value, $path)));
            assert_eq!(actual, $expected, "{}: {} expected to be: {} but was: {}", $error, $path, $expected, actual);
         )*
    }};

    ($value:expr=> $( $path:literal = $expected:expr, $error:literal)*) => {
        {
            $(
                let actual = crate::common::get_json_path(&$value, $path);
                assert!(actual == &$expected, "{}: {} expected to be: {:?} but was: {:?}", $error, $path, $expected, actual);
             )*
        }
    }
}
