use axum::{http::StatusCode, response::IntoResponse, Json};
use serde_json::json;

#[derive(Debug)]
pub enum Error {
    ServerCreate,
    DatabaseConnection,
    DatabaseError,
    MissingCredential,
    WrongCredential,
    InvalidToken,
    MissingField,
    ImageExists,
    ImageNotFound,
    TagExists,
    TagNotFound,
    UserExists,
    UserNotFound,
    Hashing,
    Upload,
    Serialize,
    InvalidId,
    NotImplemented,
    WrongType,
}

impl IntoResponse for Error {
    fn into_response(self) -> axum::response::Response {
        let (status, message) = match self {
            Error::ServerCreate => (StatusCode::INTERNAL_SERVER_ERROR, "Server Creation"),
            Error::DatabaseConnection => (StatusCode::INTERNAL_SERVER_ERROR, "Database Connection"),
            Error::DatabaseError => (StatusCode::INTERNAL_SERVER_ERROR, "Database Error"),
            Error::MissingCredential => (StatusCode::BAD_REQUEST, "Missing Credential"),
            Error::WrongCredential => (StatusCode::UNAUTHORIZED, "Wrong Credential"),
            Error::InvalidToken => (StatusCode::BAD_REQUEST, "Invalid Token"),
            Error::MissingField => (StatusCode::BAD_REQUEST, "Missing Field"),
            Error::ImageExists => (StatusCode::BAD_REQUEST, "Image already exists"),
            Error::ImageNotFound => (StatusCode::BAD_REQUEST, "Image not found"),
            Error::TagExists => (StatusCode::BAD_REQUEST, "Tag already exists"),
            Error::TagNotFound => (StatusCode::BAD_REQUEST, "Tag not found"),
            Error::UserExists => (StatusCode::BAD_REQUEST, "User already exists"),
            Error::UserNotFound => (StatusCode::BAD_REQUEST, "User not found"),
            Error::Hashing => (StatusCode::INTERNAL_SERVER_ERROR, "Hashing password"),
            Error::Upload => (StatusCode::BAD_REQUEST, "Upload Error"),
            Error::Serialize => (StatusCode::INTERNAL_SERVER_ERROR, "Serialize"),
            Error::InvalidId => (StatusCode::BAD_REQUEST, "Invalid Id"),
            Error::NotImplemented => (StatusCode::INTERNAL_SERVER_ERROR, "Not Implemented"),
            Error::WrongType => (StatusCode::BAD_REQUEST, "Wrong Type"),
        };

        let body = Json(json!({
            "error": message,
        }));

        (status, body).into_response()
    }
}

impl From<surrealdb::Error> for Error {
    fn from(e: surrealdb::Error) -> Self {
        println!("{:?}", e);
        Self::DatabaseError 
    }
}
