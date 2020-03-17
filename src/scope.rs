use super::keywords;
use super::schema;
use hashbrown::HashMap;
use simd_json::value::{Value as ValueTrait};

use super::helpers;

#[derive(Debug)]
pub struct Scope<V>
where
    V: ValueTrait,
{
    keywords: keywords::KeywordMap<V>,
    schemes: HashMap<String, schema::Schema<V>>,
}

impl<V: 'static> Scope<V>
where
    V: ValueTrait + std::clone::Clone + std::convert::From<simd_json::value::owned::Value> + std::fmt::Display,
    //String: std::borrow::Borrow<<V as simd_json::value::Value>::Key>,
    <V as ValueTrait>::Key: std::borrow::Borrow<str> + std::hash::Hash + Eq + std::convert::AsRef<str> + std::fmt::Debug + std::string::ToString + std::marker::Sync + std::marker::Send,
{
    pub fn new() -> Scope<V> {
        let mut scope = Scope {
            keywords: keywords::default(),
            schemes: HashMap::new(),
        };

        scope.add_keyword(vec!["format"], keywords::format::Format::new());
        scope
    }

    pub fn with_formats<F>(build_formats: F) -> Scope<V>
    where
        V: ValueTrait,
        F: FnOnce(&mut keywords::format::FormatBuilders<V>),
    {
        let mut scope = Scope {
            keywords: keywords::default(),
            schemes: hashbrown::HashMap::new(),
        };

        scope.add_keyword(
            vec!["format"],
            keywords::format::Format::with(build_formats),
        );

        scope
    }

    pub fn resolve(&self, id: &url::Url) -> Option<schema::ScopedSchema<V>>
    where
        <V as ValueTrait>::Key: std::borrow::Borrow<str> + std::hash::Hash + Eq + std::convert::AsRef<str> + std::fmt::Debug + std::string::ToString,
    {
        let (schema_path, fragment) = helpers::serialize_schema_path(id);

        let schema = self.schemes.get(&schema_path).or_else(|| {
            for (_, schema) in self.schemes.iter() {
                let internal_schema = schema.resolve(schema_path.as_ref());
                if internal_schema.is_some() {
                    return internal_schema;
                }
            }

            None
        });

        schema.and_then(|schema| match fragment {
            Some(ref fragment) => schema
                .resolve_fragment(fragment)
                .map(|schema| schema::ScopedSchema::new(self, &*schema)),
            None => Some(schema::ScopedSchema::new(self, &*schema)),
        })
    }

    pub fn compile_and_return(
        &'_ mut self,
        def: V,
        ban_unknown: bool,
    ) -> Result<schema::ScopedSchema<V>, schema::SchemaError>
    where
        <V as ValueTrait>::Key: std::borrow::Borrow<str> + std::hash::Hash + Eq + std::convert::AsRef<str> + std::fmt::Debug + std::string::ToString,
    {
        println!("IN  COMPILE AND RETURN");
        let schema = schema::compile(
            def,
            None,
            schema::CompilationSettings::new(self.keywords.clone(), ban_unknown),
        )?;
        println!("COMPILATION DONE");
        self.add_and_return(schema.id.clone().as_ref().unwrap(), schema)
    }

    #[allow(clippy::map_entry)] // allowing for the return values
    fn add_and_return(
        &mut self,
        id: &url::Url,
        schema: schema::Schema<V>,
    ) -> Result<schema::ScopedSchema<V>, schema::SchemaError>
    where
        <V as ValueTrait>::Key: std::borrow::Borrow<str> + std::hash::Hash + Eq + std::convert::AsRef<str> + std::fmt::Debug + std::string::ToString,
    {
        let (id_str, fragment) = helpers::serialize_schema_path(id);

        if fragment.is_some() {
            return Err(schema::SchemaError::WrongId);
        }

        if !self.schemes.contains_key(&id_str) {
            println!("schema {} not present so we are adding it", id_str);
            self.schemes.insert(id_str.clone(), schema);
            Ok(schema::ScopedSchema::new(self, &self.schemes[&id_str]))
        } else {
            Err(schema::SchemaError::IdConflicts)
        }
    }

    pub fn add_keyword<T>(&mut self, keys: Vec<&'static str>, keyword: T)
    where
        T: keywords::Keyword<V> + 'static,
    {
        keywords::decouple_keyword((keys, Box::new(keyword)), &mut self.keywords);
    }
}
