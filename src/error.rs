use thiserror::Error;

#[derive(Error, Debug)]
pub enum FliteError {
    #[error("Could not convert to CString: {0}")]
    InvalidString(#[from] std::ffi::NulError),
}