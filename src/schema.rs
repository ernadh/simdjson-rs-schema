use phf;
use std::error::Error;
use url;
use value_trait::*;

use std::fmt;
use std::fmt::{Display, Formatter};

use std::collections;

use super::helpers;
use super::keywords;
use super::scope;
use super::validators;

#[derive(Debug)]
pub struct Schema<V>
where
    V: Value,
{
    pub id: Option<url::Url>,
    schema: Option<url::Url>,
    original: V,
    tree: collections::BTreeMap<String, Schema<V>>,
    validators: validators::Validators<V>,
    scopes: hashbrown::HashMap<String, Vec<String>>,
}

include!(concat!(env!("OUT_DIR"), "/codegen.rs"));

pub struct CompilationSettings<V>
where
    V: Value,
{
    pub keywords: keywords::KeywordMap<V>,
    pub ban_unknown_keywords: bool,
}

impl<V> CompilationSettings<V>
where
    V: Value,
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
pub struct WalkContext<'walk> {
    pub url: &'walk url::Url,
    pub fragment: Vec<String>,
    pub scopes: &'walk mut hashbrown::HashMap<String, Vec<String>>,
}

impl<'walk> WalkContext<'walk> {
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
pub struct ScopedSchema<'scope, 'schema, V>
where
    V: Value + std::clone::Clone,
{
    scope: &'scope scope::Scope<V>,
    schema: &'schema Schema<V>,
}

impl<'scope, 'schema, V: std::clone::Clone> ScopedSchema<'scope, 'schema, V>
where
    V: Value + std::convert::From<simd_json::value::owned::Value>,
    <V as Value>::Key: std::borrow::Borrow<str>
        + std::hash::Hash
        + Eq
        + std::convert::AsRef<str>
        + std::fmt::Debug
        + std::string::ToString,
{
    pub fn new(
        scope: &'scope scope::Scope<V>,
        schema: &'schema Schema<V>,
    ) -> ScopedSchema<'scope, 'schema, V> {
        ScopedSchema {
            scope,
            schema: &schema,
        }
    }

    pub fn validate(&self, data: &V) -> validators::ValidationState {
        println!("Now in validate() with {:?}", data.as_str());
        self.schema.validate_in_scope(data, "", &self.scope)
    }

    pub fn validate_in(&self, data: &V, path: &str) -> validators::ValidationState {
        println!("Now in validate_in() with {:?} {:?}", data.as_str(), path);
        self.schema.validate_in_scope(data, path, &self.scope)
    }
}

impl<V: std::clone::Clone> Schema<V>
where
    V: Value + std::convert::From<simd_json::value::owned::Value>,
    <V as Value>::Key: std::borrow::Borrow<str>
        + std::hash::Hash
        + Eq
        + std::convert::AsRef<str>
        + std::fmt::Debug
        + std::string::ToString,
{
    fn validate_in_scope(
        &self,
        data: &V,
        path: &str,
        scope: &scope::Scope<V>,
    ) -> validators::ValidationState {
        println!("Now in validate_in_scope() with {:?} {:?}", data.as_str(), path);
        let mut state = validators::ValidationState::new();

        for validator in self.validators.iter() {
            println!(
                "ANOTHER VALIDATOR {:?} {:?} {:?}",
                data.as_str(),
                path,
                state
            );
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
        def: V,
        external_id: Option<url::Url>,
        settings: CompilationSettings<V>,
    ) -> Result<Schema<V>, SchemaError> {
        let def = helpers::convert_boolean_schema(def);

        if !def.is_object() {
            return Err(SchemaError::NotAnObject);
        }

        println!("DEF {:?}", def.as_str());

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
                println!("{:?} {:?}", key, value.as_str());
                if !value.is_object() && !value.is_array() && !value.is_bool() {
                    continue;
                }
                if FINAL_KEYS.contains(&key.as_ref()[..]) {
                    println!("{}", "it's a FINAL KEYS elem");
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
                    !NON_SCHEMA_KEYS.contains(&key.as_ref()[..]),
                )?;

                tree.insert(helpers::encode(key.as_ref()), scheme);
            }

            (tree, scopes)
        };

        let validators = Schema::compile_keywords(
            def.clone(),
            &WalkContext {
                url: &id,
                fragment: vec![],
                scopes: &mut scopes,
            },
            &settings,
        )?;

        println!("Validators count {}", validators.len());

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
        def: V,
        context: &WalkContext<'key>,
        settings: &CompilationSettings<V>,
    ) -> Result<validators::Validators<V>, SchemaError>
    where
        <V as Value>::Key: std::borrow::Borrow<str>
            + std::hash::Hash
            + Eq
            + std::convert::AsRef<str>
            + std::fmt::Debug
            + std::string::ToString,
    {
        let mut validators = vec![];
        let mut keys: hashbrown::HashSet<&str> = def
            .as_object()
            .unwrap()
            .keys()
            .map(|key| key.as_ref())
            .collect();
        let mut not_consumed = hashbrown::HashSet::new();
        println!("{} {:?}", "Compiling keywords", keys);

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
        def: V,
        context: &mut WalkContext<'_>,
        keywords: &CompilationSettings<V>,
        is_schema: bool,
    ) -> Result<Schema<V>, SchemaError> {
        println!("In COMPILE_SUB {:?} {:?}", def.as_str(), context);
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
                    if !PROPERTY_KEYS.contains(&parent_key[..])
                        && FINAL_KEYS.contains(&key.as_ref()[..])
                    {
                        continue;
                    }

                    let mut current_fragment = context.fragment.clone();
                    current_fragment.push(key.to_string().clone());

                    let is_schema = PROPERTY_KEYS.contains(&parent_key[..])
                        || !NON_SCHEMA_KEYS.contains(&key.as_ref()[..]);

                    let mut context = WalkContext {
                        url: id.as_ref().unwrap_or(context.url),
                        fragment: current_fragment,
                        scopes: context.scopes,
                    };

                    let scheme =
                        Schema::compile_sub(value.clone(), &mut context, keywords, is_schema)?;

                    tree.insert(helpers::encode(key.as_ref()), scheme);
                }
            } else if def.is_array() {
                println!("It's an array {:?}", def.as_str());
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

        println!("IS SCHEMA: {}", is_schema);
        println!("IS OBJECT: {}", def.is_object());

        let validators = if is_schema && def.is_object() {
            Schema::compile_keywords(def.clone(), context, keywords)?
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
    def: V,
    external_id: Option<url::Url>,
    settings: CompilationSettings<V>,
) -> Result<Schema<V>, SchemaError>
where
    V: Value + std::clone::Clone + std::convert::From<simd_json::value::owned::Value>,
    <V as Value>::Key: std::borrow::Borrow<str>
        + std::hash::Hash
        + Eq
        + std::convert::AsRef<str>
        + std::fmt::Debug
        + std::string::ToString,
{
    Schema::compile(def, external_id, settings)
}
