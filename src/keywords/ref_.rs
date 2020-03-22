use simd_json::value::Value as ValueTrait;
use url::Url;

use super::schema;
use super::validators;

pub struct Ref;

impl<V: 'static> super::Keyword<V> for Ref
where
    V: ValueTrait
        + std::clone::Clone
        + std::convert::From<simd_json::value::owned::Value>
        + std::fmt::Display
        + std::marker::Sync
        + std::marker::Send
        + std::cmp::PartialEq,
    <V as ValueTrait>::Key: std::borrow::Borrow<str>
        + std::hash::Hash
        + Eq
        + std::convert::AsRef<str>
        + std::fmt::Debug
        + std::string::ToString
        + std::marker::Sync
        + std::marker::Send,
{
    fn compile(&self, def: &V, ctx: &schema::WalkContext<'_>) -> super::KeywordResult<V> {
        let ref_ = keyword_key_exists!(def, "$ref");

        if ref_.is_str() {
            let url = Url::options()
                .base_url(Some(ctx.url))
                .parse(ref_.as_str().unwrap());
            match url {
                Ok(url) => Ok(Some(Box::new(validators::Ref { url }))),
                Err(_) => Err(schema::SchemaError::Malformed {
                    path: ctx.fragment.join("/"),
                    detail: "The value of $ref must be an URI-encoded JSON Pointer".to_string(),
                }),
            }
        } else {
            Err(schema::SchemaError::Malformed {
                path: ctx.fragment.join("/"),
                detail: "The value of multipleOf must be a string".to_string(),
            })
        }
    }
}
