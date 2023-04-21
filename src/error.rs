#[derive(thiserror::Error, Debug)]
pub enum PackerError {
    #[error("Failed to packed an image.")]
    FailedToPacked,
}
