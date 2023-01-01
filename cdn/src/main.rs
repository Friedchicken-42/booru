use std::{env, fs, net::SocketAddr, path::PathBuf};

use axum::{
    body::Bytes,
    extract::{multipart::Field, Multipart, Path},
    http::{StatusCode, HeaderMap, header},
    response::{IntoResponse, Response},
    routing::{get, post},
    Json, Router, Server,
};
use dotenv::dotenv;
use serde_json::json;

#[derive(Debug)]
enum Error {
    Server,
    WrongFilename,
    WrongField,
    Write,
    Read,
    Exists,
    NotFound,
}

impl IntoResponse for Error {
    fn into_response(self) -> Response {
        let (status, message) = match self {
            Error::Server => (StatusCode::INTERNAL_SERVER_ERROR, "Server Start"),
            Error::WrongFilename => (StatusCode::BAD_REQUEST, "Wrong Filename"),
            Error::WrongField => (StatusCode::BAD_REQUEST, "Wrong Field"),
            Error::Write => (StatusCode::INTERNAL_SERVER_ERROR, "Write File"),
            Error::Read => (StatusCode::BAD_REQUEST, "Read File"),
            Error::Exists => (StatusCode::BAD_REQUEST, "File already exists"),
            Error::NotFound => (StatusCode::BAD_REQUEST, "File not found"),
        };

        let body = Json(json!({
            "error": message,
        }));

        (status, body).into_response()
    }
}

fn split_string(s: String) -> (String, String, String, String) {
    let (x, rest) = s.split_at(2);
    let (y, rest) = rest.split_at(2);
    let (z, rest) = rest.split_at(2);

    (
        x.to_string(),
        y.to_string(),
        z.to_string(),
        rest.to_string(),
    )
}

fn generate_path(s: String) -> (PathBuf, String) {
    let (a, b, c, name) = split_string(s);

    let assets = match env::var("ASSETS") {
        Ok(p) => p,
        Err(_) => "./".to_string(),
    };

    let dir = PathBuf::from(assets).join(a).join(b).join(c);

    (dir, name)
}

fn save(filename: String, data: Bytes) -> Result<(), Error> {
    let (dir, name) = generate_path(filename);
    let path = dir.join(name);

    if path.exists() {
        return Err(Error::Exists);
    }

    fs::create_dir_all(dir).map_err(|_| Error::Write)?;

    fs::write(path, data).map_err(|_| Error::Write)?;

    Ok(())
}

async fn parse_field(field: Field<'_>) -> Option<(String, String, Bytes)> {
    let name = match field.name() {
        Some(n) => n.to_string(),
        None => return None,
    };

    let filename = match field.file_name() {
        Some(n) => n.to_string(),
        None => return None,
    };

    let data = match field.bytes().await {
        Ok(n) => n,
        Err(_) => return None,
    };

    Some((name, filename, data))
}

async fn add(mut multipart: Multipart) -> Result<(), Error> {
    while let Ok(Some(field)) = multipart.next_field().await {
        let Some((name, filename, data)) = parse_field(field).await else {
            continue;
        };

        if name != "file" {
            return Err(Error::WrongFilename);
        }

        let file = PathBuf::from(filename);

        let Some(string) = file.file_name() else {
            return Err(Error::WrongFilename);
        };

        if string.len() != 32 {
            return Err(Error::WrongFilename);
        }

        save(string.to_string_lossy().to_string(), data)?;

        return Ok(());
    }

    Err(Error::WrongField)
}

async fn image(Path(filename): Path<String>) -> Result<Response, Error> {
    if filename.len() != 32 {
        return Err(Error::WrongFilename);
    }

    let (dir, name) = generate_path(filename);
    let path = dir.join(name);

    if !path.exists() {
        return Err(Error::NotFound);
    }

    let data = fs::read(path).map_err(|_| Error::Read)?;

    let mut headers = HeaderMap::new();
    headers.insert(
        header::CONTENT_TYPE,
        "image/png".parse().expect("cannot parse string"),
    );

    Ok((headers, data).into_response())
}

#[tokio::main]
async fn main() -> Result<(), Error> {
    dotenv().ok();

    let app = Router::new()
        .route("/", post(add))
        .route("/:id", get(image));

    let addr = SocketAddr::from(([127, 0, 0, 1], 4000));

    Server::bind(&addr)
        .serve(app.into_make_service())
        .await
        .map_err(|_| Error::Server)?;

    Ok(())
}
