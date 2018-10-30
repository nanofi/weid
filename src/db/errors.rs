
use actix_web::{HttpResponse, error::ResponseError};

error_chain! {
    foreign_links {
        Io(std::io::Error);
    }
}

unsafe impl Sync for Error {}
impl ResponseError for Error { }
