use simd_json::value::{Value as ValueTrait};
use super::scope;

pub struct DateTime;

impl<V> super::Validator<V> for DateTime
where
    V: ValueTrait,
{
    fn validate(&self, val: &V, path: &str, scope: &scope::Scope<V>) -> super::ValidationState {
        let string = val.as_str();
    }
}
