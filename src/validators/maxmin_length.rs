use simd_json::value::Value as ValueTrait;

use super::error;
use super::scope;

#[allow(missing_copy_implementations)]
pub struct MaxLength {
    pub length: u64,
}

impl<V: 'static> super::Validator<V> for MaxLength
where
    V: ValueTrait + std::clone::Clone + std::convert::From<simd_json::value::owned::Value> + std::fmt::Display,
    <V as ValueTrait>::Key: std::borrow::Borrow<str> + std::hash::Hash + Eq + std::convert::AsRef<str> + std::fmt::Debug + std::string::ToString + std::marker::Sync + std::marker::Send,
{
    fn validate(&self, val: &V, path: &str, _scope: &scope::Scope<V>) -> super::ValidationState {
        let string = nonstrict_process!(val.as_str(), path);

        if (string.len() as u64) <= self.length {
            super::ValidationState::new()
        } else {
            val_error!(error::MaxLength {
                path: path.to_string()
            })
        }
    }
}

#[allow(missing_copy_implementations)]
pub struct MinLength {
    pub length: u64,
}

impl<V: 'static> super::Validator<V> for MinLength
where
    V: ValueTrait + std::clone::Clone + std::convert::From<simd_json::value::owned::Value> + std::fmt::Display,
    <V as ValueTrait>::Key: std::borrow::Borrow<str> + std::hash::Hash + Eq + std::convert::AsRef<str> + std::fmt::Debug + std::string::ToString + std::marker::Sync + std::marker::Send,
{
    fn validate(&self, val: &V, path: &str, _scope: &scope::Scope<V>) -> super::ValidationState {
        let string = nonstrict_process!(val.as_str(), path);

        if (string.len() as u64) >= self.length {
            super::ValidationState::new()
        } else {
            val_error!(error::MinLength {
                path: path.to_string()
            })
        }
    }
}
