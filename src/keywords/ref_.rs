use simd_json::value::{BorrowedValue as Value, Value as ValueTrait};
use url::Url;

use super::schema;
use super::validators;

pub struct Ref;

impl super::Keyword for Ref {
    fn compile(&self, def: &Value, ctx: &schema::WalkContext<'_>) -> super::KeywordResult {
        let ref_ = keyword_key_exists!(def, "$ref");

        if ref_.is_str() {
            let url = Url::options()
                .base_url(Some(ctx.url))
                .parse(ref_.as_str().unwrap());
            match url {
                Ok(url) => Ok(Some(Box::new(validators::Ref { url }))),
                Err(_) => Err(schema::SchemaError::Malformed {
                    path: ctx.fragment.join("/"),
                    detail: "The value of $ref MUST be an URI-encoded JSON Pointer".to_string(),
                }),

            }
        } else {
            Err(schema::SchemaError::Malformed {
                path: ctx.fragment.join("/"),
                detail: "The value of multipleOf MUST be a string".to_string(),
            })
        }
    }
}

