use simd_json::value::{BorrowedValue as Value, Value as ValueTrait};

use super::scope;

pub struct Ref {
    pub url: url::Url,
}

impl<V> super::Validator<V> for Ref
where
    V: ValueTrait,
{
    fn validate(&self, val: &V, path: &str, scope: &scope::Scope<V>) -> super::ValidationState {
        let schema = scope.resolve(&self.url);

        if schema.is_some() {
            schema.unwrap().validate_in(val, path)
        } else {
            let mut state = super::ValidationState::new();
            state.missing.push(self.url.clone());
            state
        }
    }
}
