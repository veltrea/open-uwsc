use crate::value::RuntimeValue;
use std::collections::HashMap;
use std::sync::{Arc, Mutex};

pub struct Environment {
    pub values: HashMap<String, RuntimeValue>,
    pub constants: std::collections::HashSet<String>,
    pub outer: Option<Arc<Mutex<Environment>>>,
}

impl Environment {
    pub fn new() -> Self {
        Self {
            values: HashMap::new(),
            constants: std::collections::HashSet::new(),
            outer: None,
        }
    }

    pub fn new_enclosed(outer: Arc<Mutex<Environment>>) -> Self {
        Self {
            values: HashMap::new(),
            constants: std::collections::HashSet::new(),
            outer: Some(outer),
        }
    }

    pub fn define(&mut self, name: String, value: RuntimeValue, is_const: bool) {
        self.values.insert(name.clone(), value);
        if is_const {
            self.constants.insert(name);
        }
    }

    pub fn assign(&mut self, name: &str, value: RuntimeValue) -> Result<bool, String> {
        if self.values.contains_key(name) {
            if self.constants.contains(name) {
                return Err(format!("Cannot reassign to CONST {}", name));
            }
            self.values.insert(name.to_string(), value);
            return Ok(true);
        }
        if let Some(ref outer) = self.outer {
            return outer.lock().unwrap().assign(name, value);
        }
        Ok(false)
    }

    pub fn get(&self, name: &str) -> Option<RuntimeValue> {
        if let Some(val) = self.values.get(name) {
            return Some(val.clone());
        }
        if let Some(ref outer) = self.outer {
            return outer.lock().unwrap().get(name);
        }
        None
    }

    pub fn get_at(env: Arc<Mutex<Environment>>, distance: usize, name: &str) -> Option<RuntimeValue> {
        Self::ancestor(env, distance).lock().unwrap().values.get(name).cloned()
    }

    pub fn assign_at(env: Arc<Mutex<Environment>>, distance: usize, name: String, value: RuntimeValue) -> Result<(), String> {
        let ancestor = Self::ancestor(env, distance);
        let mut lock = ancestor.lock().unwrap();
        if lock.constants.contains(&name) {
            return Err(format!("Cannot reassign to CONST {}", name));
        }
        lock.values.insert(name, value);
        Ok(())
    }

    pub fn dump(&self) {
        let mut current = Some(self);
        let mut depth = 0;
        while let Some(env) = current {
            println!("--- Scope Level {} ---", depth);
            for (name, val) in &env.values {
                println!("  {} = {:?}", name, val);
            }
            if let Some(ref outer) = env.outer {
                // We need to hold the lock to transition
                let _next = outer.lock().unwrap();
                println!("  (Outer scope exists)");
                break; 
            } else {
                current = None;
            }
        }
    }

    fn ancestor(mut env: Arc<Mutex<Environment>>, distance: usize) -> Arc<Mutex<Environment>> {
        for _ in 0..distance {
            let next = {
                let locked = env.lock().unwrap();
                locked.outer.clone().expect("Ancestor out of bounds")
            };
            env = next;
        }
        env
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_scoping() {
        let global = Arc::new(Mutex::new(Environment::new()));
        global.lock().unwrap().define("foo".into(), RuntimeValue::Integer(1), false);

        let local = Arc::new(Mutex::new(Environment::new_enclosed(global.clone())));
        local.lock().unwrap().define("bar".into(), RuntimeValue::Integer(2), false);

        // Get from current scope
        assert_eq!(local.lock().unwrap().get("bar").unwrap().as_i64(), 2);
        // Get from outer scope
        assert_eq!(local.lock().unwrap().get("foo").unwrap().as_i64(), 1);
        
        // Shadowing
        local.lock().unwrap().define("foo".into(), RuntimeValue::Integer(3), false);
        assert_eq!(local.lock().unwrap().get("foo").unwrap().as_i64(), 3);
        assert_eq!(global.lock().unwrap().get("foo").unwrap().as_i64(), 1);
    }

    #[test]
    fn test_depth_access() {
        let global = Arc::new(Mutex::new(Environment::new()));
        global.lock().unwrap().define("v".into(), RuntimeValue::Integer(100), false);

        let mid = Arc::new(Mutex::new(Environment::new_enclosed(global.clone())));
        let local = Arc::new(Mutex::new(Environment::new_enclosed(mid.clone())));

        // Access global from 2 levels deep
        assert_eq!(Environment::get_at(local.clone(), 2, "v").unwrap().as_i64(), 100);
        
        // Assign mid-level from local
        mid.lock().unwrap().define("m".into(), RuntimeValue::Integer(50), false);
        let _ = Environment::assign_at(local.clone(), 1, "m".into(), RuntimeValue::Integer(55));
        assert_eq!(mid.lock().unwrap().get("m").unwrap().as_i64(), 55);
    }
}
