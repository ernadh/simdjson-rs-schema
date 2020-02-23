use simd_json::value::{BorrowedValue as Value};

use super::scope;

pub struct Ref {
    pub url: url::Url,
}

impl super::Validator for Ref {
    fn validate(&self, val: &Value, path: &str, scope: &scope::Scope) -> super::ValidationState {
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
