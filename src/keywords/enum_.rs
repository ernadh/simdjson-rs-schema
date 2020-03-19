use simd_json::value::{Value as ValueTrait};

use super::super::schema;
use super::super::validators;

#[allow(missing_copy_implementations)]
pub struct Enum;
impl<V: 'static> super::Keyword<V> for Enum
where
    V: ValueTrait + std::clone::Clone + std::convert::From<simd_json::value::owned::Value> + std::fmt::Display + std::marker::Sync + std::marker::Send + std::cmp::PartialEq,
    <V as ValueTrait>::Key: std::borrow::Borrow<str> + std::hash::Hash + Eq + std::convert::AsRef<str> + std::fmt::Debug + std::string::ToString + std::marker::Sync + std::marker::Send,
{
    fn compile(&self, def: &V, ctx: &schema::WalkContext<'_>) -> super::KeywordResult<V> {
        let enum_ = keyword_key_exists!(def, "enum");

        if enum_.is_array() {
            let enum_ = enum_.as_array().unwrap();

            if enum_.is_empty() {
                return Err(schema::SchemaError::Malformed {
                    path: ctx.fragment.join("/"),
                    detail: "This array must have at least one element.".to_string(),
                });
            }

            Ok(Some(Box::new(validators::Enum {
                items: enum_.clone(),
            })))
        } else {
            Err(schema::SchemaError::Malformed {
                path: ctx.fragment.join("/"),
                detail: "The value of this keyword must be an array.".to_string(),
            })
        }
    }
}
