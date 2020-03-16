use simd_json::value::Value as ValueTrait;
use url;

use super::super::scope;

#[allow(missing_copy_implementations)]
pub struct PropertyNames {
    pub url: url::Url,
}

impl<V: 'static> super::Validator<V> for PropertyNames
where
    V: ValueTrait,
    String: std::borrow::Borrow<<V as simd_json::value::Value>::Key>,
{
    fn validate(&self, val: &V, path: &str, scope: &scope::Scope<V>) -> super::ValidationState
    where
        <V as ValueTrait>::Key: std::borrow::Borrow<str> + std::hash::Hash + Eq + std::convert::AsRef<str>,
    {
        let object = nonstrict_process!(val.as_object(), path);

        let schema = scope.resolve(&self.url);
        let mut state = super::ValidationState::new();

        if schema.is_some() {
            let schema = schema.unwrap();
            for key in object.keys() {
                let item_path = [path, ["[", key.as_ref(), "]"].join("").as_ref()].join("/");
                let val = key.clone().as_ref();
                state.append(schema.validate_in(val, item_path.as_ref()));
            }
        } else {
            state.missing.push(self.url.clone());
        }

        state
    }
}
