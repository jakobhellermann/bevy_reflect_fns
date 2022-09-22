pub fn type_name_of_val<T>(_: &T) -> &'static str {
    std::any::type_name::<T>()
}

#[macro_export]
macro_rules! reflect_function {
    ($fn:expr, ( $($args:tt)*) ) => {
        $crate::ReflectFunction {
            fn_name: $crate::reflect_function_macro::type_name_of_val(&$fn),
            signature: $crate::reflect_function!(@arg signature () [] $($args)* ,),
            f: |args| {
                let expected_arg_count = $crate::reflect_function!(@arg count () [] $($args)* ,);
                if args.len() != expected_arg_count {
                    return Err($crate::ReflectFunctionError::ArgCountMismatch { expected: expected_arg_count, got: args.len() });
                }

                let mut args_iter = args.iter_mut();
                let ret = $crate::reflect_function!(@arg call ($fn, args_iter) [] $($args)* ,);
                Ok(Box::new(ret))
            },
    }
    };

    (@signature o $ty:ty) => { ($crate::PassMode::Owned, core::any::TypeId::of::<$ty>()) };
    (@signature r $ty:ty) => { ($crate::PassMode::Ref, core::any::TypeId::of::<$ty>()) };
    (@signature m $ty:ty) => { ($crate::PassMode::RefMut, core::any::TypeId::of::<$ty>()) };
    (@signature () [$(($kind:ident $ty:ty),)*]) => { vec![$($crate::reflect_function!(@signature $kind $ty),)*] };

    (@count () [$(($kind:ident $ty:ty),)*]) => { 0 $(+ $crate::reflect_function!(@count $kind ))* };
    (@count $ignore:ident) => { 1 };


    (@call ($fn:expr, $args_iter:ident) [$(($kind:ident $ty:ty),)*]) => {
        $fn($($crate::reflect_function!(@call $kind $ty, $args_iter.next().unwrap() )),*)
    };
    (@call r $ty:ty, $arg:expr) => { $arg.downcast_ref::<$ty>()? };
    (@call m $ty:ty, $arg:expr) => { $arg.downcast_mut::<$ty>()? };
    (@call o $ty:ty, $arg:expr) => { $arg.from_reflect::<$ty>()? };


    (@arg $cb:ident ($($cx:tt)*) [$(($kind:ident $ty:ty),)*] $(,)?) => {
        $crate::reflect_function!(@$cb ($($cx)*) [$(($kind $ty),)*])
    };

    (@arg $cb:ident ($($cx:tt)*) [$(($kind:ident $ty:ty),)*] &$newty:ty, $($args:tt)* ) => {
        $crate::reflect_function!(@arg $cb ($($cx)*) [$(($kind $ty),)* (r $newty),] $($args)*)
    };
    (@arg $cb:ident ($($cx:tt)*) [$(($kind:ident $ty:ty),)*] &mut $newty:ty, $($args:tt)* ) => {
        $crate::reflect_function!(@arg $cb ($($cx)*) [$(($kind $ty),)* (m $newty),] $($args)*)
    };
    (@arg $cb:ident ($($cx:tt)*) [$(($kind:ident $ty:ty),)*] $newty:ty, $($args:tt)* ) => {
        $crate::reflect_function!(@arg $cb ($($cx)*) [$(($kind $ty),)* (o $newty ),] $($args)*)
    };
}
