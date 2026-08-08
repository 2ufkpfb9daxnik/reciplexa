//! Lowered IR verification.

use crate::continuation::{Continuation, ContinuationId};
use crate::handler::LoweredOp;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum VerifyError {
    DoubleResume(ContinuationId),
    InvalidProgram(&'static str),
}

pub fn verify_one_shot(ops: &[LoweredOp], conts: &mut [Continuation]) -> Result<(), VerifyError> {
    for op in ops {
        if let LoweredOp::Resume { cont } = op {
            let slot = conts
                .iter_mut()
                .find(|c| c.id == ContinuationId(*cont))
                .ok_or(VerifyError::InvalidProgram("unknown continuation"))?;
            slot.resume().map_err(|e| match e {
                crate::continuation::ContinuationError::DoubleResume(id) => {
                    VerifyError::DoubleResume(id)
                }
            })?;
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn detects_double_resume_in_program() {
        let ops = vec![LoweredOp::Resume { cont: 1 }, LoweredOp::Resume { cont: 1 }];
        let mut conts = vec![Continuation::new(ContinuationId(1))];
        assert!(verify_one_shot(&ops, &mut conts).is_err());
    }
}
