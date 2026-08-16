//! Recompute a derived value only when the authoring source string changes.

/// Cache keyed by the exact source buffer.
#[derive(Clone, Debug, Default)]
pub struct SourceCache<T> {
    source: Option<String>,
    value: Option<T>,
}

impl<T: Clone> SourceCache<T> {
    pub fn empty() -> Self {
        Self {
            source: None,
            value: None,
        }
    }

    pub fn with_value(source: String, value: T) -> Self {
        Self {
            source: Some(source),
            value: Some(value),
        }
    }

    /// Return a clone of the cached value, calling `f` only on a miss.
    pub fn get_or_insert_with(&mut self, src: &str, f: impl FnOnce(&str) -> T) -> T {
        let hit = self.source.as_deref() == Some(src) && self.value.is_some();
        if !hit {
            self.source = Some(src.to_string());
            self.value = Some(f(src));
        }
        self.value.clone().expect("value inserted")
    }
}

#[cfg(test)]
mod tests {
    use super::SourceCache;

    #[test]
    fn skips_work_when_source_is_unchanged() {
        let mut calls = 0;
        let mut cache = SourceCache::empty();
        let a = cache.get_or_insert_with("alpha", |_| {
            calls += 1;
            1
        });
        let b = cache.get_or_insert_with("alpha", |_| {
            calls += 1;
            99
        });
        let c = cache.get_or_insert_with("beta", |_| {
            calls += 1;
            2
        });
        assert_eq!((a, b, c, calls), (1, 1, 2, 2));
    }
}
