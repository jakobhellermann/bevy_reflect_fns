#[doc(hidden)] // implementation detail
pub mod reflect_function_macro;

use std::collections::HashMap;

use bevy_reflect::{FromReflect, Reflect};

pub enum ReflectFunctionError {
    ExpectedArgPassMode { expected: PassMode, got: PassMode },
    ArgTypeMismatch { expected: &'static str, got: String },
    ArgCountMismatch { expected: usize, got: usize },
}

pub enum ReflectArg<'a> {
    Ref(&'a dyn Reflect),
    RefMut(&'a mut dyn Reflect),
    Owned(&'a dyn Reflect),
}

impl<'a> ReflectArg<'a> {
    fn pass_mode(&self) -> PassMode {
        match self {
            ReflectArg::Ref(_) => PassMode::Ref,
            ReflectArg::RefMut(_) => PassMode::RefMut,
            ReflectArg::Owned(_) => PassMode::Owned,
        }
    }

    pub fn downcast_ref<T: Reflect>(&self) -> Result<&T, ReflectFunctionError> {
        let value = match *self {
            ReflectArg::Ref(value) => value,
            ref other => {
                return Err(ReflectFunctionError::ExpectedArgPassMode {
                    expected: PassMode::Ref,
                    got: other.pass_mode(),
                })
            }
        };
        if !value.is::<T>() {
            return Err(ReflectFunctionError::ArgTypeMismatch {
                expected: std::any::type_name::<T>(),
                got: value.type_name().to_owned(),
            });
        }

        Ok(value.downcast_ref::<T>().unwrap())
    }
    pub fn downcast_mut<T: Reflect>(&mut self) -> Result<&mut T, ReflectFunctionError> {
        let value = match self {
            ReflectArg::RefMut(value) => &mut **value,
            other => {
                return Err(ReflectFunctionError::ExpectedArgPassMode {
                    expected: PassMode::RefMut,
                    got: other.pass_mode(),
                })
            }
        };
        if !value.is::<T>() {
            return Err(ReflectFunctionError::ArgTypeMismatch {
                expected: std::any::type_name::<T>(),
                got: value.type_name().to_owned(),
            });
        }

        Ok(value.downcast_mut::<T>().unwrap())
    }
    pub fn from_reflect<T: FromReflect>(&self) -> Result<T, ReflectFunctionError> {
        let value = match *self {
            ReflectArg::Owned(value) => value,
            ref other => {
                return Err(ReflectFunctionError::ExpectedArgPassMode {
                    expected: PassMode::Owned,
                    got: other.pass_mode(),
                })
            }
        };
        if !value.is::<T>() {
            return Err(ReflectFunctionError::ArgTypeMismatch {
                expected: std::any::type_name::<T>(),
                got: value.type_name().to_owned(),
            });
        }

        Ok(T::from_reflect(value).unwrap())
    }
}

#[derive(Debug)]
pub enum PassMode {
    Ref,
    RefMut,
    Owned,
}

type RawReflectFunction =
    dyn Fn(&mut [&mut ReflectArg<'_>]) -> Result<Box<dyn Reflect>, ReflectFunctionError>;

pub struct ReflectFunction {
    pub fn_name: &'static str,
    pub pass_modes: Vec<PassMode>,
    pub f: Box<RawReflectFunction>,
}

impl std::fmt::Debug for ReflectFunction {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ReflectFunction")
            .field("f", &self.fn_name)
            .field("pass_modes", &self.pass_modes)
            .finish()
    }
}

#[derive(Debug)]
pub struct ReflectMethods {
    methods: HashMap<&'static str, ReflectFunction>,
}

impl ReflectMethods {
    pub fn from_methods(
        methods: impl IntoIterator<Item = (&'static str, ReflectFunction)>,
    ) -> Self {
        ReflectMethods {
            methods: HashMap::from_iter(methods),
        }
    }

    pub fn insert(&mut self, method: &'static str, function: ReflectFunction) {
        self.methods.insert(method, function);
    }

    pub fn get(&self, method: &str) -> Option<&ReflectFunction> {
        self.methods.get(method)
    }
}
