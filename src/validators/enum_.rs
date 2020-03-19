use value_trait::*;

use super::error;
use super::scope;

#[allow(missing_copy_implementations)]
pub struct Enum<V: Value> {
    pub items: <V as Value>::Array,
}

impl<V> super::Validator<V> for Enum<V>
where
    V: Value
        + std::clone::Clone
        + std::convert::From<simd_json::value::owned::Value>
        + std::fmt::Display
        + std::marker::Sync
        + std::marker::Send
        + std::cmp::PartialEq
        + 'static,
    <V as Value>::Key: std::borrow::Borrow<str>
        + std::hash::Hash
        + Eq
        + std::convert::AsRef<str>
        + std::fmt::Debug
        + std::string::ToString
        + std::marker::Sync
        + std::marker::Send,
{
    fn validate(&self, val: &V, path: &str, _scope: &scope::Scope<V>) -> super::ValidationState {
        let mut state = super::ValidationState::new();

        let mut contains = false;
        for value in self.items.iter() {
            if val == value {
                contains = true;
                break;
            }
        }

        if !contains {
            state.errors.push(Box::new(error::Enum {
                path: path.to_string(),
            }))
        }

        state
    }
}
