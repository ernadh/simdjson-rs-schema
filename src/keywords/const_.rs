use simd_json::value::{Value as ValueTrait};

use super::super::schema;
use super::super::validators;

#[allow(missing_copy_implementations)]
pub struct Const;
impl<V: 'static> super::Keyword<V> for Const
where
    V: ValueTrait + std::clone::Clone + std::convert::From<simd_json::value::owned::Value> + std::fmt::Display + std::marker::Sync + std::marker::Send + std::cmp::PartialEq,
    <V as ValueTrait>::Key: std::borrow::Borrow<str> + std::hash::Hash + Eq + std::convert::AsRef<str> + std::fmt::Debug + std::string::ToString + std::marker::Sync + std::marker::Send,
{
    fn compile(&self, def: &V, _ctx: &schema::WalkContext<'_>) -> super::KeywordResult<V> {
        let const_ = keyword_key_exists!(def, "const");

        Ok(Some(Box::new(validators::Const {
            item: const_.clone(),
        })))
    }
}
