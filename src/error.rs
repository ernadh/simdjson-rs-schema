use std::error::Error;
use std::fmt::Debug;
use std::any::{Any, TypeId};
//use simd_json::{Value, MutableValue, ValueBuilder};
use simd_json::{Value as ValueTrait};
use serde::{Serialize, Serializer};

pub trait GetTypeId: Any {
    fn typeid(&self) -> TypeId {
        TypeId::of::<Self>()
    }
}

impl<T: Any> GetTypeId for T {}

pub fn get_data_ptr<T: ?Sized>(d: *const T) -> *const () {
    d as *const ()
}

pub trait SimdjsonSchemaError: Error + Send + Debug + GetTypeId {
    fn get_code(&self) -> &str;
    fn get_path(&self) -> &str;
    fn get_title(&self) -> &str;
    fn get_detail(&self) -> Option<&str> {
        None
    }
}

impl dyn SimdjsonSchemaError {
    pub fn is<E: SimdjsonSchemaError>(&self) -> bool {
        self.typeid() == TypeId::of::<E>()
    }

    pub fn downcast<E: SimdjsonSchemaError>(&self) -> Option<&E> {
        if self.is::<E>() {
            unsafe { Some(&*(get_data_ptr(self) as *const E)) }
        } else {
            None
        }
    }
}

pub type SimdjsonSchemaErrors = Vec<Box<dyn SimdjsonSchemaError>>;

macro_rules! impl_basic_err {
    ($err:ty, $code:expr) => {
        impl ::std::error::Error for $err {
            fn description(&self) -> &str {
                $code
            }
        }

        impl ::std::fmt::Display for $err {
            fn fmt(&self, formatter: &mut ::std::fmt::Formatter<'_>) -> ::std::fmt::Result {
                //self.description().fmt(formatter)
                std::fmt::Display::fmt(&self.description(), formatter)
            }
        }
    };
}
macro_rules! impl_err {
    ($err:ty, $code:expr, $title:expr) => {
        impl_basic_err!($err, $code);

        impl SimdjsonSchemaError for $err {
            fn get_code(&self) -> &str {
                $code
            }
            fn get_title(&self) -> &str {
                $title
            }
            fn get_path(&self) -> &str {
                self.path.as_ref()
            }
        }
    };

    ($err:ty, $code:expr, $title:expr, +detail) => {
        impl_basic_err!($err, $code);

        impl SimdjsonSchemaError for $err {
            fn get_code(&self) -> &str {
                $code
            }
            fn get_title(&self) -> &str {
                $title
            }
            fn get_path(&self) -> &str {
                self.path.as_ref()
            }
            fn get_detail(&self) -> Option<&str> {
                Some(self.detail.as_ref())
            }
        }
    };

    ($err:ty, $code:expr, $title:expr, +opt_detail) => {
        impl_basic_err!($err, $code);

        impl SimdjsonSchemaError for $err {
            fn get_code(&self) -> &str {
                $code
            }
            fn get_title(&self) -> &str {
                $title
            }
            fn get_path(&self) -> &str {
                self.path.as_ref()
            }
            fn get_detail(&self) -> Option<&str> {
                self.detail.as_ref().map(|s| s.as_ref())
            }
        }
    };
}

/*
macro_rules! impl_serialize {
    ($err:ty) => {
        impl Serialize for $err {
            fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error> where S: Serializer {
                let mut map = simd_json::value::deserialize("".as_bytes_mut()).unwrap();
                //let mut map = ValueTrait::object();
                map.insert("code".to_string(), self.get_code()).unwrap();
                map.insert("title".to_string(), self.get_title()).unwrap();
                map.insert("path".to_string(), self.get_path()).unwrap();

                if let Some(detail) = self.get_detail() {
                    map.insert("detail".to_string(), detail).unwrap();
                }

                return map.serialize(serializer);
            }
        }
    };
    ($err:ty, $($sp:expr),+) => {
        impl Serialize for $err {
            fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error> where S: Serializer {
                let mut map = ValueTrait::map();
                map.insert("code".to_string(), self.get_code()).unwrap();
                map.insert("title".to_string(), self.get_title()).unwrap();
                map.insert("path".to_string(), self.get_path()).unwrap();

                if let Some(detail) = self.get_detail() {
                    map.insert("detail".to_string(), detail);
                }

                $({
                    let closure = $sp;
                    closure(self, &mut map);
                })+
                map.serialize(serializer);
            }
        }
    }
}
*/

#[derive(Debug)]
#[allow(missing_copy_implementations)]
pub struct Properties {
    pub path: String,
    pub detail: String,
}
impl_err!(Properties, "properties", "Property conditions are not met", +detail);
//impl_serialize!(Properties);

#[derive(Debug)]
pub struct Required {
    pub path: String,
}
impl_err!(Required, "required", "This property is required");
//impl_serialize!(Required);

#[derive(Debug)]
pub struct Format {
    pub path: String,
    pub detail: String,
}
impl_err!(Format, "format", "Format is wrong", +detail);
//impl_serialize!(Format);
