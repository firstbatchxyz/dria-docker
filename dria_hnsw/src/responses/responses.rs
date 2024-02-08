use actix_web::{body::BoxBody, http::header::ContentType, HttpRequest, HttpResponse, Responder};
use serde::Serialize;
use std::iter::Map;

#[derive(Serialize)]
pub struct CustomResponse<T> {
    pub(crate) success: bool,
    pub(crate) data: T,
    pub(crate) code: u32,
}
