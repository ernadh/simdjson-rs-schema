use simd_json::value::Value as ValueTrait;

use super::scope;

pub struct Ref {
    pub url: url::Url,
}

impl<V> super::Validator<V> for Ref
where
    V: ValueTrait + 'static,
{
    fn validate(&self, val: &V, path: &str, scope: &scope::Scope<V>) -> super::ValidationState
    where
        <V as ValueTrait>::Key: std::borrow::Borrow<str> + std::hash::Hash + Eq,
    {
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
