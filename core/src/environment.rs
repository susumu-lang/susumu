//! Environment for variable and function scoping in Susumu

use crate::ast::FunctionDef;
use crate::error::{SusumuError, SusumuResult};
use dashmap::DashMap;
use parking_lot::RwLock;
use serde_json::Value;
use std::sync::Arc;

/// Variable entry tracking value and mutability
#[derive(Debug, Clone)]
pub struct VariableEntry {
    pub value: Value,
    pub is_mutable: bool,
}

/// Thread-safe environment for concurrent arrow processing
#[derive(Debug, Clone)]
pub struct Environment {
    variables: Arc<DashMap<String, VariableEntry>>,
    functions: Arc<DashMap<String, FunctionDef>>,
    parent: Option<Arc<Environment>>,
}

impl Environment {
    /// Create a new empty environment
    pub fn new() -> Self {
        Self {
            variables: Arc::new(DashMap::new()),
            functions: Arc::new(DashMap::new()),
            parent: None,
        }
    }

    /// Create a new environment with a parent scope
    pub fn with_parent(parent: Arc<Environment>) -> Self {
        Self {
            variables: Arc::new(DashMap::new()),
            functions: Arc::new(DashMap::new()),
            parent: Some(parent),
        }
    }

    /// Define a variable in this environment (immutable by default)
    pub fn define(&self, name: String, value: Value) {
        self.define_with_mutability(name, value, false);
    }

    /// Define a variable with explicit mutability
    pub fn define_with_mutability(&self, name: String, value: Value, is_mutable: bool) {
        let entry = VariableEntry { value, is_mutable };
        self.variables.insert(name, entry);
    }

    /// Define a function in this environment
    pub fn define_function(&self, name: String, func: FunctionDef) {
        self.functions.insert(name, func);
    }

    /// Get a variable value, checking parent scopes if needed
    pub fn get(&self, name: &str) -> SusumuResult<Value> {
        if let Some(entry) = self.variables.get(name) {
            return Ok(entry.value.clone());
        }

        if let Some(parent) = &self.parent {
            return parent.get(name);
        }

        Err(SusumuError::undefined_variable(name))
    }

    /// Check if a variable is mutable
    pub fn is_mutable(&self, name: &str) -> SusumuResult<bool> {
        if let Some(entry) = self.variables.get(name) {
            return Ok(entry.is_mutable);
        }

        if let Some(parent) = &self.parent {
            return parent.is_mutable(name);
        }

        Err(SusumuError::undefined_variable(name))
    }

    /// Update a mutable variable
    pub fn update_mutable(&self, name: &str, new_value: Value) -> SusumuResult<()> {
        if let Some(mut entry) = self.variables.get_mut(name) {
            if entry.is_mutable {
                entry.value = new_value;
                return Ok(());
            } else {
                return Err(SusumuError::runtime_error(&format!(
                    "Cannot mutate immutable variable '{}'",
                    name
                )));
            }
        }

        if let Some(parent) = &self.parent {
            return parent.update_mutable(name, new_value);
        }

        Err(SusumuError::undefined_variable(name))
    }

    /// Get a function definition, checking parent scopes if needed
    pub fn get_function(&self, name: &str) -> SusumuResult<FunctionDef> {
        if let Some(func) = self.functions.get(name) {
            return Ok(func.clone());
        }

        if let Some(parent) = &self.parent {
            return parent.get_function(name);
        }

        Err(SusumuError::undefined_function(name))
    }

    /// Set a variable value (for updates, checking mutability)
    pub fn set(&self, name: &str, value: Value) -> SusumuResult<()> {
        // Use update_mutable which already handles mutability checks
        self.update_mutable(name, value)
    }

    /// Check if a variable exists in this environment or parent scopes
    pub fn contains_variable(&self, name: &str) -> bool {
        self.variables.contains_key(name)
            || self
                .parent
                .as_ref()
                .map_or(false, |p| p.contains_variable(name))
    }

    /// Check if a function exists in this environment or parent scopes
    pub fn contains_function(&self, name: &str) -> bool {
        self.functions.contains_key(name)
            || self
                .parent
                .as_ref()
                .map_or(false, |p| p.contains_function(name))
    }

    /// Get all variable names in this environment (for debugging)
    pub fn variable_names(&self) -> Vec<String> {
        self.variables
            .iter()
            .map(|entry| entry.key().clone())
            .collect()
    }

    /// Get all function names in this environment (for debugging)
    pub fn function_names(&self) -> Vec<String> {
        self.functions
            .iter()
            .map(|entry| entry.key().clone())
            .collect()
    }
}

impl Default for Environment {
    fn default() -> Self {
        Self::new()
    }
}

/// Production-ready environment manager for the Susumu runtime
/// Uses RwLock for thread-safe concurrent access while maintaining performance
#[derive(Debug)]
pub struct EnvironmentManager {
    global: Arc<Environment>,
    current: RwLock<Arc<Environment>>,
}

/// Thread-safe clone implementation for parallel processing
impl Clone for EnvironmentManager {
    fn clone(&self) -> Self {
        let current = self.current.read().clone();
        Self {
            global: self.global.clone(),
            current: RwLock::new(current),
        }
    }
}

impl EnvironmentManager {
    /// Create a new environment manager with global scope
    pub fn new() -> Self {
        let global = Arc::new(Environment::new());
        let current = RwLock::new(global.clone());

        Self { global, current }
    }

    /// Get the global environment
    pub fn global(&self) -> Arc<Environment> {
        self.global.clone()
    }

    /// Get the current environment
    pub fn current(&self) -> Arc<Environment> {
        self.current.read().clone()
    }

    /// Push a new scope (create child environment)
    pub fn push_scope(&self) -> Arc<Environment> {
        let current = self.current.read().clone();
        let new_scope = Arc::new(Environment::with_parent(current));
        *self.current.write() = new_scope.clone();
        new_scope
    }

    /// Pop the current scope (return to parent)
    pub fn pop_scope(&self) -> SusumuResult<()> {
        let current = self.current.read().clone();
        if let Some(parent) = &current.parent {
            *self.current.write() = parent.clone();
            Ok(())
        } else {
            Err(SusumuError::runtime_error(
                "Cannot pop scope: already at global scope",
            ))
        }
    }

    /// Execute a function with a new scope, automatically cleaning up
    pub fn with_new_scope<F, R>(&self, f: F) -> SusumuResult<R>
    where
        F: FnOnce(&Arc<Environment>) -> SusumuResult<R>,
    {
        let new_scope = self.push_scope();
        let result = f(&new_scope);
        self.pop_scope()?;
        result
    }
}

impl Default for EnvironmentManager {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_environment_variable_scoping() {
        let parent = Arc::new(Environment::new());
        parent.define("x".to_string(), json!(42));

        let child = Environment::with_parent(parent);
        child.define("y".to_string(), json!(24));

        // Child can access parent variables
        assert_eq!(child.get("x").unwrap(), json!(42));
        // Child can access own variables
        assert_eq!(child.get("y").unwrap(), json!(24));
    }

    #[test]
    fn test_environment_manager_scoping() {
        let manager = EnvironmentManager::new();

        // Define in global scope
        manager
            .global()
            .define("global_var".to_string(), json!("global"));

        // Test with new scope
        let result = manager
            .with_new_scope(|env| {
                env.define("local_var".to_string(), json!("local"));

                // Can access both global and local
                assert_eq!(env.get("global_var").unwrap(), json!("global"));
                assert_eq!(env.get("local_var").unwrap(), json!("local"));

                Ok(json!("success"))
            })
            .unwrap();

        assert_eq!(result, json!("success"));

        // Local variable should not be accessible in global scope
        assert!(manager.current().get("local_var").is_err());
        assert_eq!(
            manager.current().get("global_var").unwrap(),
            json!("global")
        );
    }
}
