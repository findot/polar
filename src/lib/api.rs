use std::io::Cursor;

use rocket::http::{ContentType, Status};
use rocket::request::Request;
use rocket::response::{Responder, Response, Result as ResponseResult};
use rocket::serde::{
    json::serde_json::{self, json},
    Serialize,
};

use crate::lib::result::Error as ApiError;

// Api response definition

#[derive(Debug)]
pub struct ApiResponse<T: Serialize> {
    response: Option<T>,
    status: Status,
}

impl<T: Serialize> ApiResponse<T> {
    #[inline]
    fn new(response: T, status: Status) -> Self {
        Self {
            response: Some(response),
            status,
        }
    }

    #[inline]
    fn ok(response: T) -> Self {
        Self::new(response, Status::Ok)
    }

    #[inline]
    fn empty(status: Status) -> Self {
        Self {
            response: None,
            status,
        }
    }

    #[inline]
    fn created(response: T) -> Self {
        Self::new(response, Status::Created)
    }

    #[inline]
    fn accepted(response: T) -> Self {
        Self::new(response, Status::Accepted)
    }

    #[inline]
    fn no_content() -> Self {
        Self::empty(Status::NoContent)
    }
}

#[rocket::async_trait]
impl<'r, 'a: 'r, T: Serialize> Responder<'r, 'a> for ApiResponse<T> {
    fn respond_to(self, _: &'r Request<'_>) -> ResponseResult<'a> {
        serde_json::to_string(&self.response)
            .map(|body| {
                Response::build()
                    .header(ContentType::JSON)
                    .status(self.status)
                    .sized_body(body.len(), Cursor::new(body))
                    .finalize()
            })
            .map_err(|_| Status::InternalServerError)
    }
}

// Api error definition

fn error_status(error: ApiError) -> Status {
    match error {
        // TODO - Add error types
        ApiError::NotFound => Status::NotFound,
        _ => Status::InternalServerError,
    }
}

#[rocket::async_trait]
impl<'r, 'a: 'r> Responder<'r, 'a> for ApiError<'a> {
    fn respond_to(self, _: &'r Request<'_>) -> ResponseResult<'a> {
        let body = json!({ "reason": format!("{}", &self) }).to_string();
        Response::build()
            .header(ContentType::JSON)
            .status(error_status(self))
            .sized_body(body.len(), Cursor::new(body))
            .ok()
    }
}
