use simd_json::value::{BorrowedValue as Value, Value as ValueTrait};
use hashbrown::HashMap;

use super::schema;
use super::validators;

pub type FormatBuilders<'key, V> = HashMap<String, Box<dyn super::Keyword<'key, V> + 'static + Send + Sync>>;

fn default_formats<'key, V>() -> FormatBuilders<'key, V> 
where
    V: ValueTrait,
    <V as ValueTrait>::Key: std::borrow::Borrow<&'key str> + std::hash::Hash + Eq,
{
    let mut map: FormatBuilders<V> = HashMap::new();

    let date_time_builder = Box::new(|_def: &Value, _ctx: &schema::WalkContext<'_>| {
        Ok(Some(
            Box::new(validators::formats::DateTime) as validators::BoxedValidator<V>
        ))
    });
    map.insert("date-time".to_string(), date_time_builder);

    map
}

pub struct Format<'key, V> {
    pub formats: FormatBuilders<'key, V>,
}

impl<'key, V> Format<'key, V>
where
    V: ValueTrait,
    <V as ValueTrait>::Key: std::borrow::Borrow<&'key str> + std::hash::Hash + Eq,
{
    pub fn new() -> Format<'key, V> {
        Format {
            formats: default_formats(),
        }
    }

    pub fn with<F>(build_formats: F) -> Format<'key, V>
    where
        F: FnOnce(&mut FormatBuilders<V>),
    {
        let mut formats = default_formats();
        build_formats(&mut formats);
        Format { formats }
    }
}

impl<'key, V> super::Keyword<'key, V> for Format<'key, V>
where
    V: ValueTrait,
    <V as ValueTrait>::Key: std::borrow::Borrow<&'key str> + std::hash::Hash + Eq,
{
    fn compile(&self, def: &Value, ctx: &schema::WalkContext<'_>) -> super::KeywordResult<'key, V> {
        let format = keyword_key_exists!(def, "format");

        if format.as_str().is_some() {
            let format = format.as_str().unwrap();
            match self.formats.get(format) {
                Some(keyword) => keyword.compile(def, ctx),
                None => Ok(None),
            }
        } else {
            Err(schema::SchemaError::Malformed {
                path: ctx.fragment.join("/"),
                detail: "The value of format must be a string".to_string(),
            })
        }
    }
}
