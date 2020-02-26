use simd_json::value::{Value as ValueTrait};

use super::scope;
use super::error;

pub struct Required {
    pub items: Vec<String>,
}

impl<'key, V> super::Validator<'key, V> for Required
where
    V: ValueTrait,
    <V as ValueTrait>::Key: std::borrow::Borrow<&'key str> + std::hash::Hash + Eq,
    <V as ValueTrait>::Key: std::borrow::Borrow<String> + std::hash::Hash + Eq,
{
    fn validate(&self, val: &V, path: &str, _scope: &scope::Scope<'key, V>) -> super::ValidationState {
        println!("IN VALIDATE for required");
        let mut state = super::ValidationState::new();

        for key in self.items.iter() {
            if val.get(key).is_none() {
                state.errors.push(Box::new(error::Required {
                    path: [path, key.as_ref()].join("/"),
                }))
            }
        }

        state
    }
}
