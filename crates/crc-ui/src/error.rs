pub type Result<T> = std::result::Result<T, UiError>;

#[derive(Debug, thiserror::Error)]
pub enum UiError {
    #[error("no GPU adapter is available")]
    NoAdapter,

    #[error("the GPU device could not be created")]
    Device(#[source] wgpu::RequestDeviceError),

    #[error("the window surface could not be created")]
    Surface(#[source] wgpu::CreateSurfaceError),
}
