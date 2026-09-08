use thiserror::Error;

#[derive(Debug, Error)]
pub enum ParseError {
    #[error("unexpected token at line {line}: {message}")]
    UnexpectedToken { line: usize, message: String },
    #[error("unknown gate: {0}")]
    UnknownGate(String),
    #[error("undefined register: {0}")]
    UndefinedRegister(String),
    #[error("index out of bounds: {register}[{index}] (size {size})")]
    IndexOutOfBounds {
        register: String,
        index: usize,
        size: usize,
    },
    #[error("invalid parameter: {0}")]
    InvalidParameter(String),
    #[error("{0}")]
    Other(String),
}
