# bevy_reflect_fns

Experimentation for trying out ways to support method calls using `bevy_reflect`.
This will be used together with `bevy_mod_js_scripting` to see if the design works out, and then hopefully upstreamed into `bevy_reflect`.

The core data this crates revolves around is this

```rust
pub enum PassMode {
    Ref,
    RefMut,
    Owned,
}

pub enum ReflectArg<'a> {
    Ref(&'a dyn Reflect),
    RefMut(&'a mut dyn Reflect),
    Owned(&'a dyn Reflect),
}

type RawReflectFunction =
    fn(&mut [&mut ReflectArg<'_>]) -> Result<Box<dyn Reflect>, ReflectFunctionError>;

pub struct ReflectFunction {
    pub fn_name: &'static str,
    pub pass_modes: Vec<PassMode>,
    pub f: RawReflectFunction,
}

pub struct ReflectMethods {
    methods: HashMap<&'static str, ReflectFunction>,
}
```

together with a macro to simplify creating such `ReflectFunction`s: `reflect_function!(Vec3::lerp: (Vec3, Vec3, f32))`.
