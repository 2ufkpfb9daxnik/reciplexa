//! Type-system identity types.

use core::fmt;

/// Resolved type identity within a module.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct TypeId(pub u64);

impl TypeId {
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

impl fmt::Display for TypeId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "type:{}", self.0)
    }
}

/// Constructor / data constructor identity.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct ConstructorId(pub u64);

impl ConstructorId {
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

impl fmt::Display for ConstructorId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "constructor:{}", self.0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn type_and_constructor_ids_are_opaque() {
        assert!(TypeId::new(1).is_valid());
        assert!(ConstructorId::new(2).is_valid());
    }

    #[test]
    fn type_id_boundaries_and_display() {
        assert!(!TypeId::INVALID.is_valid());
        assert_eq!(TypeId::INVALID.get(), 0);
        assert_eq!(TypeId::new(0).get(), 0);
        assert!(!TypeId::new(0).is_valid());
        assert!(TypeId::new(1).is_valid());
        assert!(TypeId::new(u64::MAX / 2).is_valid());
        assert!(TypeId::new(u64::MAX).is_valid());
        assert_eq!(TypeId::new(42).to_string(), "type:42");
        assert!(TypeId::new(1) < TypeId::new(2));
        assert_eq!(format!("{:?}", TypeId::new(7)), "TypeId(7)");
    }

    #[test]
    fn constructor_id_boundaries_and_display() {
        assert!(!ConstructorId::INVALID.is_valid());
        assert_eq!(ConstructorId::INVALID.get(), 0);
        assert!(!ConstructorId::new(0).is_valid());
        assert!(ConstructorId::new(1).is_valid());
        assert!(ConstructorId::new(u64::MAX).is_valid());
        assert_eq!(ConstructorId::new(9).to_string(), "constructor:9");
        assert!(ConstructorId::new(1) < ConstructorId::new(2));
    }
}
