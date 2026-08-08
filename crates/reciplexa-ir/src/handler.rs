//! Handler frames in lowered IR.

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum HandlerKind {
    Return,
    Failure,
    Op(String),
}

#[derive(Debug, Clone)]
pub struct HandlerFrame {
    pub kind: HandlerKind,
    pub handler_pc: usize,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum LoweredOp {
    Perform { op: String },
    Resume { cont: u64 },
    Raise { tag: String },
    Return,
}
