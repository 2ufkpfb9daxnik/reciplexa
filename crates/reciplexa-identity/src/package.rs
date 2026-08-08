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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn invalid_sentinel_is_zero() {
        assert!(!PackageId::INVALID.is_valid());
        assert!(!PackageInstanceId::INVALID.is_valid());
        assert!(!ModuleId::INVALID.is_valid());
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
