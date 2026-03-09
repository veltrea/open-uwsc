use super::{Builtins, BuiltinModule, BuiltinFunc, RuntimeValue};
use std::collections::HashMap;

pub struct StubsBuiltins;

impl BuiltinModule for StubsBuiltins {
    fn register(&self, r: &mut HashMap<String, BuiltinFunc>) {
        // Placeholder for unimplemented functions
        r.insert("SOUND".into(), stub_not_implemented);
        r.insert("GETSTR".into(), stub_not_implemented);
    }
}

fn stub_not_implemented(_b: &mut Builtins, _args: &mut [RuntimeValue]) -> Result<RuntimeValue, String> {
    Err("This function is not yet implemented".into())
}
