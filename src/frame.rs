use std::io::Cursor;
use std::sync::Arc;

use rocket::{response, Request, Response};
use rocket::http::ContentType;
use rocket::response::Responder;

#[derive(Clone)]
pub struct Frame {
    width: u32,
    height: u32,
    scale: u32,
    data: Arc<[u8]>,
}

impl Frame {
    pub fn encode(width: u32, height: u32, scale: u32, data: &[u8]) -> Self {
        Self {
            width,
            height,
            scale,
            data: data.into(),
        }
    }
}

impl<'r> Responder<'r, 'static> for Frame {
    fn respond_to(self, _req: &'r Request<'_>) -> response::Result<'static> {
        Response::build()
            .header(ContentType::Binary)
            .raw_header("X-Width", self.width.to_string())
            .raw_header("X-Height", self.height.to_string())
            .raw_header("X-Scale", self.scale.to_string())
            .sized_body(self.data.len(), Cursor::new(Arc::clone(&self.data)))
            .ok()
    }
}
