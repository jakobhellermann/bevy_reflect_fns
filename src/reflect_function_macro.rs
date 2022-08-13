use bevy_reflect::{FromReflect, Reflect};

use crate::{PassMode, ReflectArg, ReflectFunctionError};

pub fn type_name_of_val<T>(_: &T) -> &'static str {
    std::any::type_name::<T>()
}

#[macro_export]
macro_rules! reflect_function {
    ($fn:path: ($($param_ty:ty),*)) => {{
        use $crate::reflect_function_macro::SpecializationBaseCase;
        $crate::ReflectFunction {
            fn_name: $crate::reflect_function_macro::type_name_of_val(&$fn),
            signature: {
                vec![$(($crate::reflect_function_macro::CheckPassMode::<$param_ty>::PASS_MODE, std::any::TypeId::of::<$param_ty>())),*]
            },
            f: |args| {
                let expected_arg_count = $crate::reflect_function!(@count $($param_ty,)*);
                if args.len() != expected_arg_count {
                    return Err($crate::ReflectFunctionError::ArgCountMismatch { expected: expected_arg_count, got: args.len() });
                }

                let mut args_iter = args.iter_mut();

                let ret = $fn(
                    $(($crate::reflect_function_macro::CheckPassMode::<$param_ty>::EXTRACT_FN)(args_iter.next().unwrap())?.0),*
                );

                Ok(Box::new(ret))
            },
        }
    }};

    (@count ) => {0usize};
    (@count $head:ty, $($tail:ty,)*) => {1usize + $crate::reflect_function!(@count $($tail,)*)};
}

// helper for fake specialization
pub trait SpecializationBaseCase<T> {
    const PASS_MODE: PassMode;
    const EXTRACT_FN: fn(&mut ReflectArg<'_>) -> Result<T, ReflectFunctionError>;
}
impl<T: FromReflect> SpecializationBaseCase<T> for T {
    const PASS_MODE: PassMode = PassMode::Owned;
    const EXTRACT_FN: fn(&mut ReflectArg<'_>) -> Result<T, ReflectFunctionError> =
        |r| r.from_reflect();
}

pub struct CheckPassMode<T>(pub T);
impl<T: Reflect> CheckPassMode<&T> {
    pub const PASS_MODE: PassMode = PassMode::Ref;
    pub const EXTRACT_FN: for<'a> fn(
        &'a mut ReflectArg<'_>,
    ) -> Result<CheckPassMode<&'a T>, ReflectFunctionError> =
        |r| r.downcast_ref().map(CheckPassMode);
}
impl<T: Reflect> CheckPassMode<&mut T> {
    pub const PASS_MODE: PassMode = PassMode::RefMut;
    pub const EXTRACT_FN: for<'a> fn(
        &'a mut ReflectArg<'_>,
    )
        -> Result<CheckPassMode<&'a mut T>, ReflectFunctionError> =
        |r| r.downcast_mut().map(CheckPassMode);
}

impl<T: FromReflect> FromReflect for CheckPassMode<T> {
    fn from_reflect(reflect: &dyn Reflect) -> Option<Self> {
        T::from_reflect(reflect).map(CheckPassMode)
    }
}

// just necessary for the `CheckIsRef: FromReflect` type
impl<T: Reflect> Reflect for CheckPassMode<T> {
    fn type_name(&self) -> &str {
        self.0.type_name()
    }

    fn get_type_info(&self) -> &'static bevy_reflect::TypeInfo {
        self.0.get_type_info()
    }

    fn into_any(self: Box<Self>) -> Box<dyn std::any::Any> {
        Box::new(self.0)
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self.0.as_any()
    }

    fn as_any_mut(&mut self) -> &mut dyn std::any::Any {
        self.0.as_any_mut()
    }

    fn as_reflect(&self) -> &dyn Reflect {
        self.0.as_reflect()
    }

    fn as_reflect_mut(&mut self) -> &mut dyn Reflect {
        self.0.as_reflect_mut()
    }

    fn apply(&mut self, value: &dyn Reflect) {
        self.0.apply(value)
    }

    fn set(&mut self, value: Box<dyn Reflect>) -> Result<(), Box<dyn Reflect>> {
        self.0.set(value)
    }

    fn reflect_ref(&self) -> bevy_reflect::ReflectRef {
        self.0.reflect_ref()
    }

    fn reflect_mut(&mut self) -> bevy_reflect::ReflectMut {
        self.0.reflect_mut()
    }

    fn clone_value(&self) -> Box<dyn Reflect> {
        self.0.clone_value()
    }
}
