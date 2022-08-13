use std::any::TypeId;
use std::ops::DivAssign;

use bevy_reflect_fns::{reflect_function, PassMode, ReflectArg, ReflectFunction, ReflectMethods};
use glam::Vec3;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let methods = ReflectMethods::from_methods([
        ("normalize", reflect_function!(Vec3::normalize, (Vec3))),
        ("lerp", reflect_function!(Vec3::lerp, (Vec3, Vec3, f32))),
        (
            "lerp",
            reflect_function!(Vec3::any_orthonormal_pair, (&Vec3)),
        ),
        (
            "div_assign",
            reflect_function!(<Vec3 as DivAssign<f32>>::div_assign, (&mut Vec3, f32)),
        ),
    ]);

    let original = Vec3::new(2.0, 0.0, 0.0);
    let normalized = methods
        .get("normalize")
        .unwrap()
        .call(&mut [ReflectArg::Owned(&original)])?;

    let normalized: Vec3 = *normalized.downcast().unwrap();
    println!("{original:?}.normalize() == {normalized:?}");

    Ok(())
}

#[allow(dead_code)]
fn manual() -> ReflectFunction {
    ReflectFunction {
        fn_name: bevy_reflect_fns::reflect_function_macro::type_name_of_val(&Vec3::lerp),
        signature: vec![
            (PassMode::Owned, TypeId::of::<Vec3>()),
            (PassMode::Owned, TypeId::of::<Vec3>()),
            (PassMode::Owned, TypeId::of::<f32>()),
        ],
        f: |args| {
            let [a, b, c]: &mut [_; 3] = args.try_into().unwrap();
            let a = a.from_reflect()?;
            let b = b.from_reflect()?;
            let c = c.from_reflect()?;
            let ret = Vec3::lerp(a, b, c);

            Ok(Box::new(ret))
        },
    }
}
