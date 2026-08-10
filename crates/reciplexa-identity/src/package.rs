//! Package and module identity.

use core::fmt;

/// Identifier for a package in the dependency graph.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct PackageId(pub u64);

impl PackageId {
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

impl fmt::Display for PackageId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "package:{}", self.0)
    }
}

/// Identifier for a resolved package instance (version + configuration snapshot).
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct PackageInstanceId(pub u64);

impl PackageInstanceId {
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

impl fmt::Display for PackageInstanceId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "package-instance:{}", self.0)
    }
}

/// Identifier for a module within a package instance.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct ModuleId(pub u64);

impl ModuleId {
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

impl fmt::Display for ModuleId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "module:{}", self.0)
    }
}

/// Identifier for a top-level definition within a module (MOD-001 §20.3).
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct DefinitionId(pub u64);

impl DefinitionId {
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

impl fmt::Display for DefinitionId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "definition:{}", self.0)
    }
}

/// Identifier for a named module signature (MOD-001 §20.7).
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct SignatureId(pub u64);

impl SignatureId {
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

impl fmt::Display for SignatureId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "signature:{}", self.0)
    }
}

/// Identifier for a functor declaration (MOD-001 §20.8).
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct FunctorId(pub u64);

impl FunctorId {
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

impl fmt::Display for FunctorId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "functor:{}", self.0)
    }
}

/// Allocates monotonically increasing package ids.
#[derive(Debug, Clone, Default)]
pub struct PackageIdAllocator {
    next: u64,
}

impl PackageIdAllocator {
    pub fn new() -> Self {
        Self { next: 1 }
    }

    pub fn allocate(&mut self) -> PackageId {
        let id = PackageId::new(self.next);
        self.next = self.next.saturating_add(1);
        id
    }
}

/// Allocates package instance ids.
#[derive(Debug, Clone, Default)]
pub struct PackageInstanceIdAllocator {
    next: u64,
}

impl PackageInstanceIdAllocator {
    pub fn new() -> Self {
        Self { next: 1 }
    }

    pub fn allocate(&mut self) -> PackageInstanceId {
        let id = PackageInstanceId::new(self.next);
        self.next = self.next.saturating_add(1);
        id
    }
}

/// Allocates module ids within a package instance.
#[derive(Debug, Clone, Default)]
pub struct ModuleIdAllocator {
    next: u64,
}

impl ModuleIdAllocator {
    pub fn new() -> Self {
        Self { next: 1 }
    }

    pub fn allocate(&mut self) -> ModuleId {
        let id = ModuleId::new(self.next);
        self.next = self.next.saturating_add(1);
        id
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn invalid_sentinel_is_zero() {
        assert!(!PackageId::INVALID.is_valid());
        assert!(!PackageInstanceId::INVALID.is_valid());
        assert!(!ModuleId::INVALID.is_valid());
        assert!(!DefinitionId::INVALID.is_valid());
        assert!(!SignatureId::INVALID.is_valid());
        assert!(!FunctorId::INVALID.is_valid());
    }

    #[test]
    fn allocators_issue_monotonic_ids() {
        let mut pkg = PackageIdAllocator::new();
        let a = pkg.allocate();
        let b = pkg.allocate();
        assert!(a.get() < b.get());
        let _ = PackageIdAllocator::default();

        let mut inst = PackageInstanceIdAllocator::new();
        assert!(inst.allocate().is_valid());
        assert_eq!(inst.allocate().get(), 2);
        let _ = PackageInstanceIdAllocator::default();

        let mut mod_alloc = ModuleIdAllocator::new();
        assert_eq!(mod_alloc.allocate().get(), 1);
        let _ = ModuleIdAllocator::default();
    }

    #[test]
    fn display_formats_ids() {
        assert_eq!(PackageId::new(7).to_string(), "package:7");
        assert_eq!(PackageInstanceId::new(3).to_string(), "package-instance:3");
        assert_eq!(PackageInstanceId::new(3).get(), 3);
        assert_eq!(ModuleId::new(9).to_string(), "module:9");
        assert_eq!(DefinitionId::new(4).to_string(), "definition:4");
        assert_eq!(SignatureId::new(5).to_string(), "signature:5");
        assert_eq!(FunctorId::new(6).to_string(), "functor:6");
        assert!(DefinitionId::new(1).is_valid());
        assert!(SignatureId::new(1).is_valid());
        assert!(FunctorId::new(1).is_valid());
    }
}
