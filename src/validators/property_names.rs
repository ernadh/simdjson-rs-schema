use simd_json::value::Value as ValueTrait;
use url;

use super::super::scope;

#[allow(missing_copy_implementations)]
pub struct PropertyNames {
    pub url: url::Url,
}

impl<V: 'static> super::Validator<V> for PropertyNames
where
    V: ValueTrait + std::clone::Clone + std::convert::From<simd_json::value::owned::Value> + std::fmt::Display,
    //String: std::borrow::Borrow<<V as simd_json::value::Value>::Key>,
    <V as ValueTrait>::Key: std::borrow::Borrow<str> + std::hash::Hash + Eq + std::convert::AsRef<str> + std::fmt::Debug + std::string::ToString + std::marker::Sync + std::marker::Send,
{
    fn validate(&self, val: &V, path: &str, scope: &scope::Scope<V>) -> super::ValidationState
    {
        let object = nonstrict_process!(val.as_object(), path);

        let schema = scope.resolve(&self.url);
        let mut state = super::ValidationState::new();

        if schema.is_some() {
            let schema = schema.unwrap();
            for key in object.keys() {
                let item_path = [path, ["[", key.as_ref(), "]"].join("").as_ref()].join("/");
                // NOTE: Quite likely needing actual key thing here.
                let v = object.get(key.as_ref()).unwrap();
                state.append(schema.validate_in(v, item_path.as_ref()));
            }
        } else {
            state.missing.push(self.url.clone());
        }

        state
    }
}
