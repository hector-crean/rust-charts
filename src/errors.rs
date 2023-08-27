///ThrandError  enumerates all possible errors returned by this library.
#[derive(thiserror::Error, Debug)]
pub enum ChartAppErrors {
    #[error(transparent)]
    IoError(#[from] std::io::Error),
}

pub type Result<T> = color_eyre::eyre::Result<T, ChartAppErrors>;
