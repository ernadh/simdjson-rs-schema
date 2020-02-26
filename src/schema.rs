use phf;
use simd_json::value::{BorrowedValue as Value, Value as ValueTrait};
use std::error::Error;
use url;

use std::fmt;
use std::fmt::{Display, Formatter};

use std::collections;
use std::marker::PhantomData;

use super::helpers;
use super::keywords;
use super::scope;
use super::validators;

#[derive(Debug)]
pub struct Schema<V>
where
    V: ValueTrait,
{
    pub id: Option<url::Url>,
    schema: Option<url::Url>,
    original: Value<'static>,
    tree: collections::BTreeMap<String, Schema<V>>,
    validators: validators::Validators<V>,
    scopes: hashbrown::HashMap<String, Vec<String>>,
}

include!(concat!(env!("OUT_DIR"), "/codegen.rs"));

pub struct CompilationSettings<V>
where
    V: ValueTrait,
{
    pub keywords: keywords::KeywordMap<V>,
    pub ban_unknown_keywords: bool,
}

impl<V> CompilationSettings<V>
where
    V: ValueTrait,
{
    pub fn new(
        keywords: keywords::KeywordMap<V>,
        ban_unknown_keywords: bool,
    ) -> CompilationSettings<V> {
        CompilationSettings {
            keywords,
            ban_unknown_keywords,
        }
    }
}

#[derive(Debug)]
pub struct WalkContext<'a> {
    pub url: &'a url::Url,
    pub fragment: Vec<String>,
    pub scopes: &'a mut hashbrown::HashMap<String, Vec<String>>,
}

impl<'a> WalkContext<'a> {
    pub fn escaped_fragment(&self) -> String {
        helpers::connect(
            self.fragment
                .iter()
                .map(|s| s.as_ref())
                .collect::<Vec<&str>>()
                .as_ref(),
        )
    }
}

#[derive(Debug)]
#[allow(missing_copy_implementations)]
pub enum SchemaError {
    WrongId,
    IdConflicts,
    NotAnObject,
    UrlParseError(url::ParseError),
    UnknownKey(String),
    Malformed { path: String, detail: String },
}

impl Display for SchemaError {
    fn fmt(&self, f: &mut Formatter) -> Result<(), fmt::Error> {
        match *self {
            SchemaError::WrongId => write!(f, "wrong id"),
            SchemaError::IdConflicts => write!(f, "id conflicts"),
            SchemaError::NotAnObject => write!(f, "not an object"),
            SchemaError::UrlParseError(ref e) => write!(f, "url parse error: {}", e),
            SchemaError::UnknownKey(ref k) => write!(f, "unknown key: {}", k),
            SchemaError::Malformed {
                ref path,
                ref detail,
            } => write!(f, "malformed path: `{}`, details: {}", path, detail),
        }
    }
}

impl Error for SchemaError {}

#[derive(Debug)]
pub struct ScopedSchema<V>
where
    V: ValueTrait,
{
    scope: scope::Scope<V>,
    schema: Schema<V>,
}

impl<V> ScopedSchema<V>
where
    V: ValueTrait,
    <V as ValueTrait>::Key: std::borrow::Borrow<str> + std::hash::Hash + Eq,
{
    pub fn new(scope: scope::Scope<V>, schema: Schema<V>) -> ScopedSchema<V> {
        ScopedSchema { scope, schema }
    }

    pub fn validate(&self, data: &V) -> validators::ValidationState {
        self.schema.validate_in_scope(data, "", &self.scope)
    }

    pub fn validate_in(&self, data: &V, path: &str) -> validators::ValidationState {
        self.schema.validate_in_scope(data, path, &self.scope)
    }
}

impl<V> Schema<V>
where
    V: ValueTrait,
    <V as ValueTrait>::Key: std::borrow::Borrow<str> + std::hash::Hash + Eq,
{
    fn validate_in_scope(
        &self,
        data: &V,
        path: &str,
        scope: &scope::Scope<V>,
    ) -> validators::ValidationState {
        let mut state = validators::ValidationState::new();

        for validator in self.validators.iter() {
            println!("ANOTHER VALIDATOR");
            state.append(validator.validate(data, path, scope))
        }

        state
    }

    pub fn resolve(&self, id: &str) -> Option<&Schema<V>> {
        let path = self.scopes.get(id);
        path.map(|path| {
            let mut schema = self;
            for item in path.iter() {
                schema = &schema.tree[item]
            }
            schema
        })
    }

    pub fn resolve_fragment(&self, fragment: &str) -> Option<&Schema<V>> {
        assert!(fragment.starts_with('/'), "Can't resolve id fragments");

        let parts = fragment[1..].split('/');
        let mut schema = self;
        for part in parts {
            match schema.tree.get(part) {
                Some(sch) => schema = sch,
                None => return None,
            }
        }

        Some(schema)
    }

    fn compile(
        def: Value<'static>,
        external_id: Option<url::Url>,
        settings: CompilationSettings<V>,
    ) -> Result<Schema<V>, SchemaError> {
        let def = helpers::convert_boolean_schema(def);

        if !def.is_object() {
            return Err(SchemaError::NotAnObject);
        }

        let id = if external_id.is_some() {
            external_id.unwrap()
        } else {
            helpers::parse_url_key("$id", &def)?
                .clone()
                .unwrap_or_else(helpers::generate_id)
        };

        let schema = helpers::parse_url_key("$schema", &def)?;

        let (tree, mut scopes) = {
            let mut tree = collections::BTreeMap::new();
            let obj = def.as_object().unwrap();

            let mut scopes = hashbrown::HashMap::new();

            for (key, value) in obj.iter() {
                if !value.is_object() && !value.is_array() && !value.is_bool() {
                    continue;
                }
                if FINAL_KEYS.contains(&key[..]) {
                    continue;
                }

                let mut context = WalkContext {
                    url: &id,
                    fragment: vec![key.to_string().clone()],
                    scopes: &mut scopes,
                };

                let scheme = Schema::compile_sub(
                    value.clone(),
                    &mut context,
                    &settings,
                    !NON_SCHEMA_KEYS.contains(&key[..]),
                )?;

                tree.insert(helpers::encode(key), scheme);
            }

            (tree, scopes)
        };

        let validators = Schema::compile_keywords(
            def,
            &WalkContext {
                url: &id,
                fragment: vec![],
                scopes: &mut scopes,
            },
            &settings,
        )?;

        let schema = Schema {
            id: Some(id),
            schema,
            original: def,
            tree,
            validators,
            scopes,
        };

        Ok(schema)
    }

    fn compile_keywords<'key>(
        def: Value<'static>,
        context: &WalkContext<'_>,
        settings: &CompilationSettings<V>,
    ) -> Result<validators::Validators<V>, SchemaError>
    where
        <V as ValueTrait>::Key: std::borrow::Borrow<str> + std::hash::Hash + Eq,
    {
        let mut validators = vec![];
        let mut keys: hashbrown::HashSet<&str> = def
            .as_object()
            .unwrap()
            .keys()
            .map(|key| key.as_ref())
            .collect();
        let mut not_consumed = hashbrown::HashSet::new();

        loop {
            let key = keys.iter().next().cloned();
            if key.is_some() {
                let key = key.unwrap();
                match settings.keywords.get(&key) {
                    Some(keyword) => {
                        keyword.consume(&mut keys);

                        let is_exclusive_keyword = keyword.keyword.is_exclusive();

                        if let Some(validator) = keyword.keyword.compile(&def, context)? {
                            if is_exclusive_keyword {
                                validators = vec![validator];
                            } else {
                                validators.push(validator);
                            }
                        }

                        if is_exclusive_keyword {
                            break;
                        }
                    }
                    None => {
                        keys.remove(&key);
                        if settings.ban_unknown_keywords {
                            not_consumed.insert(key);
                        }
                    }
                }
            } else {
                break;
            }
        }

        if settings.ban_unknown_keywords && !not_consumed.is_empty() {
            for key in not_consumed.iter() {
                if !ALLOW_NON_CONSUMED_KEYS.contains(&key[..]) {
                    return Err(SchemaError::UnknownKey(key.to_string()));
                }
            }
        }

        Ok(validators)
    }
    fn compile_sub(
        def: Value<'static>,
        context: &mut WalkContext<'_>,
        keywords: &CompilationSettings<V>,
        is_schema: bool,
    ) -> Result<Schema<V>, SchemaError> {
        let def = helpers::convert_boolean_schema(def);

        let id = if is_schema {
            helpers::parse_url_key_with_base("$id", &def, context.url)?
        } else {
            None
        };

        let schema = if is_schema {
            helpers::parse_url_key("$schema", &def)?
        } else {
            None
        };

        let tree = {
            let mut tree = collections::BTreeMap::new();

            if def.is_object() {
                let obj = def.as_object().unwrap();
                let parent_key = &context.fragment[context.fragment.len() - 1];

                for (key, value) in obj.iter() {
                    if !value.is_object() && !value.is_array() && !value.is_bool() {
                        continue;
                    }
                    if !PROPERTY_KEYS.contains(&parent_key[..]) && FINAL_KEYS.contains(&key[..]) {
                        continue;
                    }

                    let mut current_fragment = context.fragment.clone();
                    current_fragment.push(key.to_string().clone());

                    let is_schema = PROPERTY_KEYS.contains(&parent_key[..])
                        || !NON_SCHEMA_KEYS.contains(&key[..]);

                    let mut context = WalkContext {
                        url: id.as_ref().unwrap_or(context.url),
                        fragment: current_fragment,
                        scopes: context.scopes,
                    };

                    let scheme =
                        Schema::compile_sub(value.clone(), &mut context, keywords, is_schema)?;

                    tree.insert(helpers::encode(key), scheme);
                }
            } else if def.is_array() {
                let array = def.as_array().unwrap();
                let parent_key = &context.fragment[context.fragment.len() - 1];

                for (idx, value) in array.iter().enumerate() {
                    let mut value = value.clone();

                    if BOOLEAN_SCHEMA_ARRAY_KEYS.contains(&parent_key[..]) {
                        value = helpers::convert_boolean_schema(value);
                    }

                    if !value.is_object() && !value.is_array() {
                        continue;
                    }

                    let mut current_fragment = context.fragment.clone();
                    current_fragment.push(idx.to_string().clone());

                    let mut context = WalkContext {
                        url: id.as_ref().unwrap_or(context.url),
                        fragment: current_fragment,
                        scopes: context.scopes,
                    };

                    let scheme = Schema::compile_sub(value.clone(), &mut context, keywords, true)?;

                    tree.insert(idx.to_string().clone(), scheme);
                }
            }

            tree
        };

        if id.is_some() {
            context
                .scopes
                .insert(id.clone().unwrap().into_string(), context.fragment.clone());
        }

        let validators = if is_schema && def.is_object() {
            Schema::compile_keywords(def, context, keywords)?
        } else {
            vec![]
        };

        let schema = Schema {
            id,
            schema,
            original: def,
            tree,
            validators,
            scopes: hashbrown::HashMap::new(),
        };

        Ok(schema)
    }
}

pub fn compile<V>(
    def: Value<'static>,
    external_id: Option<url::Url>,
    settings: CompilationSettings<V>,
) -> Result<Schema<V>, SchemaError>
where
    V: ValueTrait,
    <V as ValueTrait>::Key: std::borrow::Borrow<str> + std::hash::Hash + Eq,
{
    Schema::compile(def, external_id, settings)
}
