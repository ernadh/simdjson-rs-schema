use std::error::Error;
use std::fmt::Debug;
use std::any::{Any, TypeId};
use simd_json::{to_borrowed_value as to_value, BorrowedValue as Value, Value as ValueTrait, MutableValue, ValueBuilder};
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
                self.description().fmt(formatter)
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

macro_rules! impl_serialize {
    ($err:ty) => {
        impl Serialize for $err {
            fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error> where S: Serializer {
                let mut map = Value::object();
                map.insert("code".to_string(), to_value(self.get_code().as_bytes_mut()).unwrap());
                map.insert("title".to_string(), to_value(self.get_title().as_bytes_mut()).unwrap());
                map.insert("path".to_string(), to_value(self.get_path().as_bytes_mut()).unwrap());
                if let Some(ref detail) = self.get_detail() {
                    map.insert("detail".to_string(), to_value(detail.as_bytes_mut()).unwrap());
                }
                //Value::Object(Box::new(map.as_str().unwrap().as_bytes_mut())).serialize(serializer)
                Value::Object(Box::new(*map.as_object().unwrap())).serialize(serializer)
            }
        }
    };
    ($err:ty, $($sp:expr),+) => {
        impl Serialize for $err {
            fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error> where S: Serializer {
                let mut map = simd_json::Value::map();
                map.insert("code".to_string(), to_value(self.get_code()).unwrap());
                map.insert("title".to_string(), to_value(self.get_title()).unwrap());
                map.insert("path".to_string(), to_value(self.get_path()).unwrap());
                if let Some(ref detail) = self.get_detail() {
                    map.insert("detail".to_string(), to_value(detail).unwrap());
                }
                $({
                    let closure = $sp;
                    closure(self, &mut map);
                })+
                Value::Object(map).serialize(serializer)
            }
        }
    }
}

#[derive(Debug)]
pub struct Required {
    pub path: String,
}
impl_err!(Required, "required", "This property is required");
impl_serialize!(Required);
