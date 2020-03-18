use super::error;
use super::scope;
use super::primitive_types;
use simd_json::value::Value as ValueTrait;

use std::fmt;

macro_rules! nonstrict_process {
    ($val:expr, $path:ident) => {{
        let maybe_val = $val;
        if maybe_val.is_none() {
            return $crate::validators::ValidationState::new();
        }

        maybe_val.unwrap()
    }};
}

macro_rules! val_error {
    ($err:expr) => {
        $crate::validators::ValidationState {
            errors: vec![Box::new($err)],
            missing: vec![],
        }
    };
}

#[macro_export]
macro_rules! strict_process {
    ($val:expr, $path:ident, $err:expr) => {{
        let maybe_val = $val;
        if maybe_val.is_none() {
            return val_error!($crate::error::WrongType {
                path: $path.to_string(),
                detail: $err.to_string()
            });
        }

        maybe_val.unwrap()
    }};
}

pub use self::ref_::Ref;
pub use self::required::Required;
pub use self::properties::Properties;
pub use self::property_names::PropertyNames;
pub use self::pattern::Pattern;
pub use self::unique_items::UniqueItems;
pub use self::maxmin::{ExclusiveMaximum, ExclusiveMinimum, Maximum, Minimum};
pub use self::maxmin_items::{MaxItems, MinItems};
pub use self::maxmin_length::{MaxLength, MinLength};
pub use self::maxmin_properties::{MaxProperties, MinProperties};
pub use self::type_::Type;
pub use self::of::{AllOf, AnyOf, OneOf};
pub use self::multiple_of::MultipleOf;
pub use self::not::Not;

pub mod formats;
pub mod ref_;
pub mod required;
pub mod properties;
pub mod property_names;
pub mod pattern;
pub mod type_;
mod maxmin;
mod maxmin_items;
mod maxmin_length;
mod maxmin_properties;
pub mod unique_items;
pub mod of;
pub mod multiple_of;
pub mod not;

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

    pub fn is_valid(&self) -> bool {
        self.errors.is_empty()
    }

    pub fn append(&mut self, second: ValidationState) {
        self.errors.extend(second.errors);
        self.missing.extend(second.missing);
    }
}

pub trait Validator<V>
where
    V: ValueTrait,
{
    fn validate(&self, item: &V, _: &str, _: &scope::Scope<V>) -> ValidationState
    where
        <V as ValueTrait>::Key: std::borrow::Borrow<str> + std::hash::Hash + Eq + std::convert::AsRef<str>;
}

impl<V> fmt::Debug for dyn Validator<V> + 'static + Send + Sync
where
    V: ValueTrait,
{
    fn fmt(&self, fmt: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt.write_str("<validator>")
    }
}

pub type BoxedValidator<V> = Box<dyn Validator<V> + 'static + Send + Sync>;
pub type Validators<V> = Vec<BoxedValidator<V>>;
