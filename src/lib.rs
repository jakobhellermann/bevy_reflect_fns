#[doc(hidden)] // implementation detail
pub mod reflect_function_macro;

use std::{any::TypeId, collections::HashMap};

use bevy_reflect::{FromReflect, Reflect};

#[derive(thiserror::Error, Debug)]
pub enum ReflectFunctionError {
    #[error("Expected arg pass mode of {expected:?} but got {got:?}.")]
    ExpectedArgPassMode { expected: PassMode, got: PassMode },
    #[error("Expected arg of type {expected} but got {got}.")]
    ArgTypeMismatch { expected: &'static str, got: String },
    #[error("Expected arg count of {expected} but got {got}.")]
    ArgCountMismatch { expected: usize, got: usize },
}

#[derive(Debug)]
pub enum ReflectArg<'a> {
    Ref(&'a dyn Reflect),
    RefMut(&'a mut dyn Reflect),
    Owned(&'a dyn Reflect),
}

impl ReflectArg<'_> {
    pub fn pass<T: Reflect>(pass_mode: PassMode, value: &mut T) -> ReflectArg<'_> {
        match pass_mode {
            PassMode::Ref => ReflectArg::Ref(value),
            PassMode::RefMut => ReflectArg::RefMut(value),
            PassMode::Owned => ReflectArg::Owned(value),
        }
    }

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

        T::from_reflect(value).ok_or_else(|| ReflectFunctionError::ArgTypeMismatch {
            expected: std::any::type_name::<T>(),
            got: value.type_name().to_owned(),
        })
    }
}

#[derive(Copy, Clone, Debug)]
pub enum PassMode {
    Ref,
    RefMut,
    Owned,
}

type RawReflectFunction =
    fn(&mut [ReflectArg<'_>]) -> Result<Box<dyn Reflect>, ReflectFunctionError>;

#[derive(Clone)]
pub struct ReflectFunction {
    pub fn_name: &'static str,
    pub signature: Vec<(PassMode, TypeId)>,
    pub f: RawReflectFunction,
}

impl ReflectFunction {
    pub fn call(
        &self,
        args: &mut [ReflectArg<'_>],
    ) -> Result<Box<dyn Reflect>, ReflectFunctionError> {
        (self.f)(args)
    }
}

impl std::fmt::Debug for ReflectFunction {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ReflectFunction")
            .field("f", &self.fn_name)
            .field("pass_modes", &self.signature)
            .finish()
    }
}

#[derive(Clone, Debug)]
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

    pub fn methods(&self) -> impl Iterator<Item = &'static str> + '_ {
        self.methods.keys().copied()
    }

    pub fn get(&self, method: &str) -> Option<&ReflectFunction> {
        self.methods.get(method)
    }
}
