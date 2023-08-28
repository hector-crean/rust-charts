use crate::models::EnumIntConversionError;

///ThrandError  enumerates all possible errors returned by this library.
#[derive(thiserror::Error, Debug)]
pub enum ChartAppErrors {
    #[error(transparent)]
    IoError(#[from] std::io::Error),
    #[error(transparent)]
    SerdeJsonError(#[from] serde_json::Error),
    #[error(transparent)]
    CsvError(#[from] csv::Error),
    #[error(transparent)]
    JsonReadError(#[from] crate::file_op::JsonReadError),
    #[error(transparent)]
    EnumIntConversionError(#[from] EnumIntConversionError),
}

pub type Result<T> = color_eyre::eyre::Result<T, ChartAppErrors>;
