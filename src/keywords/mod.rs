use std::any;
use std::fmt;
use std::sync::Arc;

use hashbrown::HashMap;
use simd_json::value::{Value as ValueTrait};

use super::schema;
use super::validators;
use super::helpers;

pub type KeywordPair<V> = (Vec<&'static str>, Box<dyn Keyword<V> + 'static>);
pub type KeywordResult<V> = Result<Option<validators::BoxedValidator<V>>, schema::SchemaError>;
pub type KeywordMap<V> = HashMap<&'static str, Arc<KeywordConsumer<V>>>;

pub trait Keyword<V>: Send + Sync + any::Any
where
    V: ValueTrait + std::clone::Clone + std::convert::From<simd_json::value::owned::Value>,
{
    fn compile(&self, def: &V, ctx: &schema::WalkContext) -> KeywordResult<V>
    where
        <V as ValueTrait>::Key: std::borrow::Borrow<str> + std::hash::Hash + Eq;
    fn is_exclusive(&self) -> bool {
        false
    }
}

impl<T: 'static + Send + Sync + any::Any, V> Keyword<V> for T
where
    V: ValueTrait + std::clone::Clone + std::convert::From<simd_json::value::owned::Value>,
    T: Fn(&V, &schema::WalkContext<'_>) -> KeywordResult<V>,
{
    fn compile(&self, def: &V, ctx: &schema::WalkContext<'_>) -> KeywordResult<V>
    where
        <V as ValueTrait>::Key: std::borrow::Borrow<str> + std::hash::Hash + Eq,
    {
        self(def, ctx)
    }
}

impl<V> fmt::Debug for dyn Keyword<V> + 'static {
    fn fmt(&self, fmt: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt.write_str("<keyword>")
    }
}

macro_rules! keyword_key_exists {
    ($val:expr, $key:expr) => {{
        let maybe_val = $val.get($key);

        if maybe_val.is_none() {
            return Ok(None);
        } else {
            maybe_val.unwrap()
        }
    }};
}

#[macro_use]
pub mod maxmin_length;
pub mod format;
pub mod properties;
pub mod property_names;
pub mod ref_;
pub mod required;
pub mod pattern;
pub mod type_;
pub mod unique_items;
pub mod of;
pub mod multiple_of;
pub mod not;
pub mod maxmin;
pub mod maxmin_items;
pub mod maxmin_properties;

pub fn default<V: 'static>() -> KeywordMap<V>
where
    V: ValueTrait + std::clone::Clone + std::convert::From<simd_json::value::owned::Value> + std::fmt::Display,
    <V as ValueTrait>::Key: std::borrow::Borrow<str> + std::hash::Hash + Eq + std::convert::AsRef<str> + std::fmt::Debug + std::string::ToString + std::marker::Sync + std::marker::Send,
{
    let mut map = HashMap::new();

    decouple_keyword((vec!["$ref"], Box::new(ref_::Ref)), &mut map);
    decouple_keyword((vec!["allOf"], Box::new(of::AllOf)), &mut map);
    decouple_keyword((vec!["anyOf"], Box::new(of::AnyOf)), &mut map);
    decouple_keyword((vec!["oneOf"], Box::new(of::OneOf)), &mut map);
    decouple_keyword((vec!["multipleOf"], Box::new(multiple_of::MultipleOf)), &mut map);
    decouple_keyword((vec!["not"], Box::new(not::Not)), &mut map);
    decouple_keyword((vec!["required"], Box::new(required::Required)), &mut map);
    decouple_keyword((vec!["type"], Box::new(type_::Type)), &mut map);
    decouple_keyword((vec!["exclusiveMaximum"], Box::new(maxmin::ExclusiveMaximum)), &mut map);
    decouple_keyword((vec!["exclusiveMinimum"], Box::new(maxmin::ExclusiveMinimum)), &mut map);
    decouple_keyword((vec!["maxItems"], Box::new(maxmin_items::MaxItems)), &mut map);
    decouple_keyword((vec!["maxLength"], Box::new(maxmin_length::MaxLength)), &mut map);
    decouple_keyword((vec!["maxProperties"], Box::new(maxmin_properties::MaxProperties)), &mut map);
    decouple_keyword((vec!["maximum"], Box::new(maxmin::Maximum)), &mut map);
    decouple_keyword((vec!["minItems"], Box::new(maxmin_items::MinItems)), &mut map);
    decouple_keyword((vec!["minLength"], Box::new(maxmin_length::MinLength)), &mut map);
    decouple_keyword((vec!["minProperties"], Box::new(maxmin_properties::MinProperties)), &mut map);
    decouple_keyword((vec!["minimum"], Box::new(maxmin::Minimum)), &mut map);
    decouple_keyword(
        (vec!["uniqueItems"], Box::new(unique_items::UniqueItems)),
        &mut map,
    );
    decouple_keyword((vec!["pattern"], Box::new(pattern::Pattern)), &mut map);
    decouple_keyword(
        (
            vec!["properties", "additionalProperties", "patternProperties"],
            Box::new(properties::Properties),
        ),
        &mut map,
    );
    decouple_keyword(
        (
            vec!["propertyNames"],
            Box::new(property_names::PropertyNames),
        ),
        &mut map,
    );

    map
}

#[derive(Debug)]
pub struct KeywordConsumer<V>
where
    V: ValueTrait,
{
    pub keys: Vec<&'static str>,
    pub keyword: Box<dyn Keyword<V> + 'static>,
}

impl<V> KeywordConsumer<V>
where
    V: ValueTrait,
{
    pub fn consume(&self, set: &mut hashbrown::HashSet<&str>) {
        for key in self.keys.iter() {
            if set.contains(key) {
                set.remove(key);
            }
        }
    }
}

pub fn decouple_keyword<V>(keyword_pair: KeywordPair<V>, map: &mut KeywordMap<V>)
where
    V: ValueTrait,
{
    let (keys, keyword) = keyword_pair;

    let consumer = Arc::new(KeywordConsumer {
        keys: keys.clone(),
        keyword,
    });

    for key in keys.iter() {
        map.insert(key, consumer.clone());
    }
}
