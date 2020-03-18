use simd_json::{Value as ValueTrait};

use super::super::helpers;
use super::super::schema;
use super::super::validators;

#[allow(missing_copy_implementations)]
pub struct PropertyNames;
impl<V: 'static> super::Keyword<V> for PropertyNames
where
    V: ValueTrait + std::clone::Clone + std::convert::From<simd_json::value::owned::Value> + std::fmt::Display,
    <V as ValueTrait>::Key: std::borrow::Borrow<str> + std::hash::Hash + Eq + std::convert::AsRef<str> + std::fmt::Debug + std::string::ToString + std::marker::Sync + std::marker::Send,
{
    fn compile(&self, def: &V, ctx: &schema::WalkContext<'_>) -> super::KeywordResult<V> {
        let property_names = def.get("propertyNames").unwrap();


        if property_names.is_object() || property_names.is_bool() {
            Ok(Some(Box::new(validators::PropertyNames {
                url: helpers::alter_fragment_path(
                    ctx.url.clone(),
                    [ctx.escaped_fragment().as_ref(), "propertyNames"].join("/"),
                ),
            })))
        } else {
            Err(schema::SchemaError::Malformed {
                path: ctx.fragment.join("/"),
                detail: "The value of propertyNames must be an object or a boolean".to_string(),
            })
        }
    }
}
