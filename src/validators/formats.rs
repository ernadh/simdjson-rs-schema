use simd_json::value::{Value as ValueTrait};
use super::scope;
use chrono;

use super::error;
#[allow(missing_copy_implementations)]
pub struct DateTime;

impl<'key, V> super::Validator<'key, V> for DateTime
where
    V: ValueTrait,
    <V as ValueTrait>::Key: std::borrow::Borrow<&'key str> + std::hash::Hash + Eq,
{
    fn validate(&self, val: &V, path: &str, _scope: &scope::Scope<'key, V>) -> super::ValidationState {
        // instead of nonstrict_process
        let string = val.as_str().unwrap();

        match chrono::DateTime::parse_from_rfc3339(string) {
            Ok(_) => super::ValidationState::new(),
            Err(_) => val_error!(error::Format {
                path: path.to_string(),
                detail: "Malformed date time".to_string()
            }),
        }
    }
}
