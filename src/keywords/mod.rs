use std::sync::Arc;
use std::any;
use std::fmt;

use simd_json::value::{BorrowedValue as Value};
use hashbrown::HashMap;

use super::schema;
use super::validators;

pub type KeywordPair = (Vec<&'static str>, Box<dyn Keyword + 'static>);
pub type KeywordResult = Result<Option<validators::BoxedValidator>, schema::SchemaError>;
pub type KeywordMap = HashMap<&'static str, Arc<KeywordConsumer>>;

pub trait Keyword: Send + Sync + any::Any {
    fn compile(&self, def: &Value, ctx: &schema::WalkContext) -> KeywordResult;
    fn is_exclusive(&self) -> bool {
        false
    }
}

impl<T: 'static + Send + Sync + any::Any> Keyword for T
where
    T: Fn(&Value, &schema::WalkContext<'_>) -> KeywordResult,
{
    fn compile(&self, def: &Value, ctx: &schema::WalkContext<'_>) -> KeywordResult {
        self(def, ctx)
    }
}

impl fmt::Debug for dyn Keyword + 'static {
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

pub fn default() -> KeywordMap {
    let mut map = HashMap::new();

    decouple_keyword((vec!["$ref"], Box::new(ref_::Ref)), &mut map);

    map
}

#[derive(Debug)]
pub struct KeywordConsumer {
    pub keys: Vec<&'static str>,
    pub keyword: Box<dyn Keyword + 'static>,
}

impl KeywordConsumer {
    pub fn consume(&self, set: &mut hashbrown::HashSet<&str>) {
        for key in self.keys.iter() {
            if set.contains(key) {
                set.remove(key);
            }
        }
    }
}

pub fn decouple_keyword(keyword_pair: KeywordPair, map: &mut KeywordMap) {
    let (keys, keyword) = keyword_pair;

    let consumer = Arc::new(KeywordConsumer {
        keys: keys.clone(),
        keyword
    });

    for key in keys.iter() {
        map.insert(key, consumer.clone());
    }
}
