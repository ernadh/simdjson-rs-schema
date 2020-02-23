use simd_json::value::{BorrowedValue as Value, Value as ValueTrait};
use simd_json::json;
use url::percent_encoding;
use url::Url;
use uuid::Uuid;

use super::schema;

pub fn encode(string: &str) -> String {
    percent_encoding::percent_encode(
        string
            .replace("~", "~0")
            .replace("/", "~1")
            .replace("%", "%25")
            .as_bytes(),
        percent_encoding::QUERY_ENCODE_SET,
    )
    .to_string()
}

pub fn connect(strings: &[&str]) -> String {
    strings
        .iter()
        .map(|s| encode(s))
        .collect::<Vec<String>>()
        .join("/")
}

pub fn generate_id() -> Url {
    let uuid = Uuid::new_v4();
    Url::parse(&format!("json-schema://{}", uuid)).unwrap()
}

pub fn parse_url_key(key: &str, obj: &Value) -> Result<Option<Url>, schema::SchemaError> {
    match obj.get(key) {
        Some(value) => match value.as_str() {
            Some(string) => Url::parse(string)
                .map(Some)
                .map_err(schema::SchemaError::UrlParseError),
            None => Ok(None),
        }
        None => Ok(None)
    }
}


pub fn serialize_schema_path(url: &Url) -> (String, Option<String>) {
    let mut url_without_fragment = url.clone();
    url_without_fragment.set_fragment(None);
    let mut url_str = url_without_fragment.into_string();

    match url.fragment().as_ref() {
        Some(fragment) if !fragment.is_empty() => {
            if !fragment.starts_with('/') {
                let fragment_parts = fragment
                    .split('/')
                    .map(|s| s.to_string())
                    .collect::<Vec<String>>();
                url_str.push_str("#");
                url_str.push_str(fragment_parts[0].as_ref());
                let fragment = if fragment_parts.len() > 1 {
                    Some("/".to_string() + fragment_parts[1..].join("/").as_ref())
                } else {
                    None
                };
                (url_str, fragment)
            } else {
                (url_str, Some(fragment.to_string()))
            }
        }
        _ => (url_str, None),
    }
}

pub fn convert_boolean_schema(val: Value) -> Value {
    match val.as_bool() {
        Some(b) => {
            if b {
                json!({}).into()
            } else {
                json!({"not": {}}).into()
            }
        }
        None => val,
    }
}

pub fn parse_url_key_with_base(key: &str, obj: &Value, base: &Url) -> Result<Option<Url>, schema::SchemaError> {
    match obj.get(key) {
        Some(value) => match value.as_str() {
            Some(string) => Url::options()
                .base_url(Some(base))
                .parse(string)
                .map(Some)
                .map_err(schema::SchemaError::UrlParseError),
            None => Ok(None),
        },
        None => Ok(None),
    }
}
