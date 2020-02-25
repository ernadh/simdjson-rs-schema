use simd_json::value::{BorrowedValue as Value, Value as ValueTrait, to_borrowed_value as to_value};

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
        let object = val.as_object().unwrap();
        let mut state = super::ValidationState::new();

        for key in self.items.iter() {
            let k = to_value(key.as_bytes_mut());
            if !object.contains_key(key) {
                state.errors.push(Box::new(error::Required {
                    path: [path, key.as_ref()].join("/"),
                }))
            }
        }

        state
    }
}
