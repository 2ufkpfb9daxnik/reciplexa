//! Integration tests moved from src/scope.rs for region coverage.

use reciplexa_bind::scope::*;


#[test]
fn inner_scope_shadows_outer() {
    let mut stack = ScopeStack::new();
    let outer = stack.declare("x");
    stack.push_scope();
    let inner = stack.declare("x");
    assert_eq!(stack.lookup("x"), Some(inner));
    assert_ne!(outer, inner);
    stack.pop_scope();
    assert_eq!(stack.lookup("x"), Some(outer));
}

#[test]
fn root_pop_is_noop_and_lookup_miss() {
    let mut stack = ScopeStack::new();
    assert_eq!(stack.depth(), 1);
    assert!(stack.pop_scope().is_none());
    assert!(stack.lookup("missing").is_none());
}

#[test]
fn default_stack_has_root_scope() {
    let stack = ScopeStack::default();
    assert_eq!(stack.depth(), 1);
}
