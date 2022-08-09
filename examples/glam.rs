use bevy_reflect_fns::{reflect_function, PassMode, ReflectFunction, ReflectMethods};
use glam::Vec3;

fn main() {
    let methods = ReflectMethods::from_methods([
        ("normalize", reflect_function!(Vec3::normalize: (Vec3))),
        ("lerp", reflect_function!(Vec3::lerp: (Vec3, Vec3, f32))),
        (
            "lerp",
            reflect_function!(Vec3::any_orthonormal_pair: (&Vec3)),
        ),
    ]);
    println!("{methods:#?}");
}

#[allow(dead_code)]
fn manual() -> ReflectFunction {
    ReflectFunction {
        fn_name: bevy_reflect_fns::reflect_function_macro::type_name_of_val(&Vec3::lerp),
        pass_modes: vec![PassMode::Owned],
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
