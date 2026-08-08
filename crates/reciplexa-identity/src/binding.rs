//! Binding and view identity types.

use core::fmt;

/// Resolved binding identity within a module.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct BindingId(pub u64);

impl BindingId {
    pub const INVALID: Self = Self(0);

    pub const fn new(value: u64) -> Self {
        Self(value)
    }

    pub const fn get(self) -> u64 {
        self.0
    }

    pub const fn is_valid(self) -> bool {
        self.0 != 0
    }
}

impl fmt::Display for BindingId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "binding:{}", self.0)
    }
}

/// Distinguishes multiple views of the same document.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct ViewInstanceId(pub u64);

impl ViewInstanceId {
    pub const INVALID: Self = Self(0);

    pub const fn new(value: u64) -> Self {
        Self(value)
    }

    pub const fn get(self) -> u64 {
        self.0
    }

    pub const fn is_valid(self) -> bool {
        self.0 != 0
    }
}

impl fmt::Display for ViewInstanceId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "view:{}", self.0)
    }
}

/// Allocates monotonically increasing [`BindingId`] values for a resolve session.
#[derive(Debug, Clone, Default)]
pub struct BindingIdAllocator {
    next: u64,
}

impl BindingIdAllocator {
    pub fn new() -> Self {
        Self { next: 1 }
    }

    pub fn allocate(&mut self) -> BindingId {
        let id = BindingId::new(self.next);
        self.next = self.next.saturating_add(1);
        id
    }
}

/// Allocates monotonically increasing [`ViewInstanceId`] values.
#[derive(Debug, Clone, Default)]
pub struct ViewInstanceIdAllocator {
    next: u64,
}

impl ViewInstanceIdAllocator {
    pub fn new() -> Self {
        Self { next: 1 }
    }

    pub fn allocate(&mut self) -> ViewInstanceId {
        let id = ViewInstanceId::new(self.next);
        self.next = self.next.saturating_add(1);
        id
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn allocators_issue_unique_ids() {
        let mut bindings = BindingIdAllocator::new();
        let mut views = ViewInstanceIdAllocator::new();
        assert_ne!(bindings.allocate(), bindings.allocate());
        assert_ne!(views.allocate(), views.allocate());
    }
}
