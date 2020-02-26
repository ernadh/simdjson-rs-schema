use super::scope;
use chrono;
use simd_json::value::Value as ValueTrait;

use super::error;
#[allow(missing_copy_implementations)]
pub struct DateTime;

impl<V> super::Validator<V> for DateTime
where
    V: ValueTrait,
{
    fn validate(&self, val: &V, path: &str, _scope: &scope::Scope<V>) -> super::ValidationState
    where
        <V as ValueTrait>::Key: std::borrow::Borrow<str> + std::hash::Hash + Eq,
    {
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
