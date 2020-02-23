use simd_json::value::{BorrowedValue as Value};
use super::scope;
use super::error;

use std::fmt;

pub use self::ref_::Ref;

pub mod ref_;

#[derive(Debug)]
pub struct ValidationState {
    pub errors: super::error::SimdjsonSchemaErrors,
    pub missing: Vec<url::Url>,
}

impl ValidationState {
    pub fn new() -> ValidationState {
        ValidationState {
            errors: vec![],
            missing: vec![],
        }
    }

    pub fn append(&mut self, second: ValidationState) {
        self.errors.extend(second.errors);
        self.missing.extend(second.missing);
    }
}

pub trait Validator {
    fn validate(&self, item: &Value, _: &str, _: &scope::Scope) -> ValidationState;
}

impl fmt::Debug for dyn Validator + 'static + Send + Sync {
    fn fmt(&self, fmt: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt.write_str("<validator>")
    }
}

pub type BoxedValidator = Box<dyn Validator + 'static + Send + Sync>;
pub type Validators = Vec<BoxedValidator>;
