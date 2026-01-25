use crate::consts::SCALE;

use std::io::Cursor;
use std::sync::Arc;

use rocket::{response, Request, Response};
use rocket::http::ContentType;
use rocket::response::Responder;

use opencv::prelude::*;

#[derive(Clone)]
pub struct Frame {
    pub width: i32,
    pub height: i32,
    pub data: Arc<[u8]>,
}

impl Frame {
    pub fn encode(mat: &Mat) -> Self {
        let width = mat.cols();
        let height = mat.rows();

        let bytes = mat.data_bytes().unwrap().to_vec();

        Self {
            width,
            height,
            data: bytes.into(),
        }
    }
}

impl<'r> Responder<'r, 'static> for Frame {
    fn respond_to(self, _req: &'r Request<'_>) -> response::Result<'static> {
        Response::build()
            .header(ContentType::Binary)
            .raw_header("X-Width", self.width.to_string())
            .raw_header("X-Height", self.height.to_string())
            .raw_header("X-Scale", SCALE.to_string())
            .sized_body(self.data.len(), Cursor::new(Arc::clone(&self.data)))
            .ok()
    }
}
