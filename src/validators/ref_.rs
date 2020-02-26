use simd_json::value::{Value as ValueTrait};

use super::scope;

pub struct Ref {
    pub url: url::Url,
}

impl<'key, V: 'key> super::Validator<'key, V> for Ref
where
    V: ValueTrait,
    <V as ValueTrait>::Key: std::borrow::Borrow<&'key str> + std::hash::Hash + Eq,
{
    fn validate(&self, val: &V, path: &str, scope: &scope::Scope<'key, V>) -> super::ValidationState {
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
