use simd_json::value::{BorrowedValue as Value, Value as ValueTrait};

use super::scope;
use super::error;

pub struct Required {
    pub items: Vec<String>,
}

impl<V> super::Validator<V> for Required
where
    V: ValueTrait,
{
    fn validate(&self, val: &V, path: &str, _scope: &scope::Scope<V>) -> super::ValidationState {
        let object = nonstrict_process!(val.as_object(), path);
        let mut state = super::ValidationState::new();

        for key in self.items.iter() {
            if !object.contains_key(key) {
                state.errors.push(Box::new(error::Required {
                    path: [path, key.as_ref()].join("/"),
                }))
            }
        }

        state
    }
}
