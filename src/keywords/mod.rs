use std::any;
use std::fmt;
use std::sync::Arc;

use hashbrown::HashMap;
use simd_json::value::{BorrowedValue as Value, Value as ValueTrait};

use super::schema;
use super::validators;

pub type KeywordPair<V> = (Vec<&'static str>, Box<dyn Keyword<V> + 'static>);
pub type KeywordResult<V> = Result<Option<validators::BoxedValidator<V>>, schema::SchemaError>;
pub type KeywordMap<V> = HashMap<&'static str, Arc<KeywordConsumer<V>>>;

pub trait Keyword<V>: Send + Sync + any::Any
where
    V: ValueTrait,
{
    fn compile(&self, def: &Value, ctx: &schema::WalkContext) -> KeywordResult<V>;
    fn is_exclusive(&self) -> bool {
        false
    }
}

impl<T: 'static + Send + Sync + any::Any, V: ValueTrait> Keyword<V> for T
where
    T: Fn(&Value, &schema::WalkContext<'_>) -> KeywordResult<V>,
{
    fn compile(&self, def: &Value, ctx: &schema::WalkContext<'_>) -> KeywordResult<V> {
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

pub mod ref_;

pub fn default<V>() -> KeywordMap<V>
where
    V: ValueTrait,
{
    let mut map = HashMap::new();

    decouple_keyword((vec!["$ref"], Box::new(ref_::Ref)), &mut map);

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
