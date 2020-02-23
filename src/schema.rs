use phf;
use simd_json::value::{BorrowedValue as Value, Value as ValueTrait};
use url;
use std::error::Error;

use std::fmt;
use std::fmt::{Display, Formatter};

use std::collections;

use super::helpers;
use super::scope;
use super::validators;
use super::keywords;

#[derive(Debug)]
pub struct Schema<'a> {
    pub id: Option<url::Url>,
    schema: Option<url::Url>,
    original: Value<'a>,
    tree: collections::BTreeMap<String, Schema<'a>>,
    validators: validators::Validators,
    scopes: hashbrown::HashMap<String, Vec<String>>,
}

include!(concat!(env!("OUT_DIR"), "/codegen.rs"));

pub struct CompilationSettings<'a> {
    pub keywords: &'a keywords::KeywordMap,
    pub ban_unknown_keywords: bool,
}

impl<'a> CompilationSettings<'a> {
    pub fn new(keywords: &'a keywords::KeywordMap, ban_unknown_keywords: bool) -> CompilationSettings<'a> {
        CompilationSettings {
            keywords,
            ban_unknown_keywords
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
pub struct ScopedSchema<'a> {
    scope: &'a scope::Scope<'a>,
    schema: &'a Schema<'a>,
}

impl<'a> ScopedSchema<'a> {
    pub fn new(scope: &'a scope::Scope, schema: &'a Schema) -> ScopedSchema<'a> {
        ScopedSchema { scope, schema }
    }

    pub fn validate(&self, data: &Value) -> validators::ValidationState {
        self.schema.validate_in_scope(data, "", self.scope)
    }

    pub fn validate_in(&self, data: &Value, path: &str) -> validators::ValidationState {
        self.schema.validate_in_scope(data, path, self.scope)
    }
}

impl<'a> Schema<'a> {
    fn validate_in_scope(&self, data: &Value, path: &str, scope: &scope::Scope) -> validators::ValidationState {
        let mut state = validators::ValidationState::new();

        for validator in self.validators.iter() {
            state.append(validator.validate(data, path, scope))
        }

        state
    }

    pub fn resolve(&self, id: &str) -> Option<&Schema> {
        let path = self.scopes.get(id);
        path.map(|path| {
            let mut schema = self;
            for item in path.iter() {
                schema = &schema.tree[item]
            }
            schema
        })
    }

    pub fn resolve_fragment(&self, fragment: &str) -> Option<&Schema> {
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
        def: Value<'a>,
        external_id: Option<url::Url>,
        settings: CompilationSettings<'_>,
    ) -> Result<Schema<'a>, SchemaError> {
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
            &def,
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

    fn compile_keywords(
        def: &Value<'a>,
        context: &WalkContext<'_>,
        settings: &CompilationSettings<'_>,
    ) -> Result<validators::Validators, SchemaError> {
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

                        if let Some(validator) = keyword.keyword.compile(def, context)? {
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
        def: Value<'a>,
        context: &mut WalkContext<'_>,
        keywords: &CompilationSettings<'_>,
        is_schema: bool,
    ) -> Result<Schema<'a>, SchemaError> {
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
            Schema::compile_keywords(&def, context, keywords)?
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

pub fn compile<'a>(def: Value<'a>, external_id: Option<url::Url>, settings: CompilationSettings<'_>) -> Result<Schema<'a>, SchemaError> {
    Schema::compile(def, external_id, settings)
}
