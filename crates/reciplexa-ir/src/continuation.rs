//! One-shot continuations.

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct ContinuationId(pub u64);

#[derive(Debug, Clone)]
pub struct Continuation {
    pub id: ContinuationId,
    pub resumed: bool,
}

impl Continuation {
    pub fn new(id: ContinuationId) -> Self {
        Self { id, resumed: false }
    }

    pub fn resume(&mut self) -> Result<(), ContinuationError> {
        if self.resumed {
            return Err(ContinuationError::DoubleResume(self.id));
        }
        self.resumed = true;
        Ok(())
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ContinuationError {
    DoubleResume(ContinuationId),
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn second_resume_fails() {
        let mut k = Continuation::new(ContinuationId(1));
        k.resume().unwrap();
        assert!(k.resume().is_err());
    }
}
