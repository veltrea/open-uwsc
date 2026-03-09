use super::{Builtins, BuiltinModule, BuiltinFunc, RuntimeValue};
use std::collections::HashMap;

pub struct StableBuiltins;

impl BuiltinModule for StableBuiltins {
    fn register(&self, _r: &mut HashMap<String, BuiltinFunc>) {
        // Functions considered fully stable across all categories
        // For now, most implemented functions are in their categorical modules.
    }
}
