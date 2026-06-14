use std::path::PathBuf;

pub type Result<T> = std::result::Result<T, UfoError>;

#[derive(Debug, thiserror::Error)]
pub enum UfoError {
    #[error("failed to read {path}: {source}")]
    ReadFile {
        path: PathBuf,
        source: std::io::Error,
    },
    #[error("failed to write {path}: {source}")]
    WriteFile {
        path: PathBuf,
        source: std::io::Error,
    },
    #[error("failed to parse MAD input: {0}")]
    Parse(String),
    #[error("unknown MAD element kind `{0}`")]
    UnknownElementKind(String),
    #[error("unknown element or line reference `{0}`")]
    UnknownReference(String),
    #[error("missing required parameter `{parameter}` for `{kind}`")]
    MissingParameter { kind: String, parameter: String },
    #[error("unsupported parameter `{parameter}` for `{kind}`")]
    UnsupportedParameter { kind: String, parameter: String },
    #[error("parameter `{parameter}` for `{kind}` must be numeric")]
    ExpectedNumber { kind: String, parameter: String },
    #[error("parameter `{parameter}` for `{kind}` must be a numeric vector")]
    ExpectedVector { kind: String, parameter: String },
    #[error("parameter `{parameter}` for `{kind}` must be a string")]
    ExpectedString { kind: String, parameter: String },
    #[error("bytecode instruction `{name}` uses {count} arguments, maximum is {max}")]
    TooManyInstructionArgs {
        name: String,
        count: usize,
        max: usize,
    },
    #[error(
        "bytecode instruction `{name}` needs {words} words, larger than chunk size {chunk_words}"
    )]
    InstructionTooLargeForChunk {
        name: String,
        words: usize,
        chunk_words: usize,
    },
    #[error("bytecode chunk size must be at least 2 words")]
    InvalidBytecodeChunkSize,
    #[error("tracking observation `{0}` is not supported yet")]
    UnsupportedObservation(String),
    #[cfg(feature = "opencl")]
    #[error("OpenCL error: {0}")]
    OpenCl(String),
}
