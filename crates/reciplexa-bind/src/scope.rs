//! Lexical scope tree.

use std::collections::HashMap;

use reciplexa_identity::binding::BindingId;
use reciplexa_identity::scope::{ScopeId, ScopeIdAllocator};

/// A lexical scope frame with local bindings.
#[derive(Debug, Clone)]
pub struct ScopeFrame {
    pub id: ScopeId,
    pub bindings: HashMap<String, BindingId>,
}

/// Stack of nested scopes used during resolution.
#[derive(Debug, Clone)]
pub struct ScopeStack {
    frames: Vec<ScopeFrame>,
    alloc: ScopeIdAllocator,
    next_binding: u64,
}

impl ScopeStack {
    pub fn new() -> Self {
        let mut stack = Self {
            frames: Vec::new(),
            alloc: ScopeIdAllocator::new(),
            next_binding: 1,
        };
        stack.push_scope();
        stack
    }

    pub fn push_scope(&mut self) -> ScopeId {
        let id = self.alloc.allocate();
        self.frames.push(ScopeFrame {
            id,
            bindings: HashMap::new(),
        });
        id
    }

    pub fn pop_scope(&mut self) -> Option<ScopeId> {
        if self.frames.len() <= 1 {
            return None;
        }
        self.frames.pop().map(|f| f.id)
    }

    pub fn declare(&mut self, name: impl Into<String>) -> BindingId {
        let id = BindingId::new(self.next_binding);
        self.next_binding = self.next_binding.saturating_add(1);
        if let Some(frame) = self.frames.last_mut() {
            frame.bindings.insert(name.into(), id);
        }
        id
    }

    pub fn lookup(&self, name: &str) -> Option<BindingId> {
        for frame in self.frames.iter().rev() {
            if let Some(id) = frame.bindings.get(name) {
                return Some(*id);
            }
        }
        None
    }

    pub fn depth(&self) -> usize {
        self.frames.len()
    }
}

impl Default for ScopeStack {
    fn default() -> Self {
        Self::new()
    }
}

/// Immutable snapshot of scopes for inspection.
#[derive(Debug, Clone)]
pub struct ScopeTree {
    pub root: ScopeId,
    pub frames: Vec<ScopeFrame>,
}

#[cfg(test)]
mod tests {
    use super::*;

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
}
