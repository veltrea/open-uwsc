use super::{Builtins, BuiltinModule, BuiltinFunc, RuntimeValue};
use std::collections::HashMap;

pub struct ExperimentalBuiltins;

impl BuiltinModule for ExperimentalBuiltins {
    fn register(&self, r: &mut HashMap<String, BuiltinFunc>) {
        #[cfg(feature = "experimental")]
        {
            r.insert("EXPERIMENTAL_FUNC".into(), experimental_func);
        }
    }
}

#[cfg(feature = "experimental")]
fn experimental_func(_b: &mut Builtins, _args: &mut [RuntimeValue]) -> Result<RuntimeValue, String> {
    Ok(RuntimeValue::String("This is an experimental builtin function".into()))
}
