use serde::Serialize;

#[derive(Serialize)]
pub struct CustomResponse<T> {
    pub(crate) success: bool,
    pub(crate) data: T,
    pub(crate) code: u32,
}
