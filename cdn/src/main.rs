use std::{
    env,
    io::{BufReader, Cursor},
    net::SocketAddr,
    path::PathBuf, fs,
};

use axum::{
    extract::{Multipart, Path},
    http::{header, HeaderMap, StatusCode},
    response::{IntoResponse, Response},
    routing::{get, post},
    Json, Router, Server,
};
use dotenv::dotenv;
use serde_json::json;
use thumbnailer::{create_thumbnails, Thumbnail, ThumbnailSize};

#[derive(Debug)]
enum Error {
    Server,
    WrongFilename,
    WrongField,
    WrongMime,
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
            Error::WrongMime => (StatusCode::BAD_REQUEST, "Wrong Mime"),
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

fn generate_path(root: &str, s: String) -> (PathBuf, String) {
    let (a, b, c, name) = split_string(s);

    let assets = match env::var(root) {
        Ok(p) => p,
        Err(_) => "./".to_string(),
    };

    let dir = PathBuf::from(assets).join(a).join(b).join(c);

    (dir, name)
}

fn save(root: &str, filename: String, data: &Vec<u8>) -> Result<(), Error> {
    let (dir, name) = generate_path(root, filename);
    let path = dir.join(name);

    if path.exists() {
        return Err(Error::Exists);
    }

    fs::create_dir_all(dir).map_err(|_| Error::Write)?;

    fs::write(path, data).map_err(|_| Error::Write)?;

    Ok(())
}

fn save_thumb(filename: String, thumb: Thumbnail) -> Result<(), Error> {
    let mut buf = Cursor::new(Vec::new());
    thumb.write_jpeg(&mut buf, 85).map_err(|_| Error::Write)?;
    let data = buf.into_inner();

    save("THUMBS", filename, &data)
}

async fn add(mut multipart: Multipart) -> Result<(), Error> {
    while let Ok(Some(field)) = multipart.next_field().await {
        let name = field.name().ok_or(Error::WrongField)?;

        if name != "file" {
            return Err(Error::WrongFilename);
        }

        let filename = field.file_name().ok_or(Error::WrongField)?.to_string();

        if filename.len() != 32 {
            return Err(Error::WrongFilename);
        }

        let content_type = field.content_type().ok_or(Error::WrongField)?;
        let content_type: mime::Mime = content_type.parse().map_err(|_| Error::WrongMime)?;

        let data = field.bytes().await.map_err(|_| Error::WrongField)?;

        save("ASSETS", filename.clone(), &data.to_vec())?;

        let cursor = Cursor::new(data);
        let reader = BufReader::new(cursor);
        let thumb = create_thumbnails(reader, content_type, [ThumbnailSize::Medium])
            .map_err(|_| Error::Write)?
            .pop()
            .ok_or(Error::Write)?;

        save_thumb(filename, thumb)?;

        return Ok(());
    }

    Err(Error::WrongField)
}

async fn thumb(Path(filename): Path<String>) -> Result<Response, Error> {
    if filename.len() != 32 {
        return Err(Error::WrongFilename);
    }

    let (dir, name) = generate_path("THUMBS", filename);
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

async fn image(Path(filename): Path<String>) -> Result<Response, Error> {
    if filename.len() != 32 {
        return Err(Error::WrongFilename);
    }

    let (dir, name) = generate_path("ASSETS", filename);
    let path = dir.join(name);

    if !path.exists() {
        return Err(Error::NotFound);
    }

    let data = fs::read(path).map_err(|_| Error::Read)?;

    let mut headers = HeaderMap::new();
    headers.insert(
        header::CONTENT_TYPE,
        "image/jpeg".parse().expect("cannot parse string"),
    );

    Ok((headers, data).into_response())
}

#[tokio::main]
async fn main() -> Result<(), Error> {
    dotenv().ok();

    let app = Router::new()
        .route("/", post(add))
        .route("/:id", get(image))
        .route("/thumb/:id", get(thumb));

    let addr = SocketAddr::from(([127, 0, 0, 1], 4000));

    Server::bind(&addr)
        .serve(app.into_make_service())
        .await
        .map_err(|_| Error::Server)?;

    Ok(())
}
