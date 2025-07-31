use std::path::Path;
use std::io::{self, Cursor};
use actix_web::{
    web, App, Error, HttpResponse, HttpServer, Responder,
    middleware::Logger,
    http::header::ContentDisposition,
};
use actix_multipart::Multipart;
use futures_util::StreamExt as _;
use image::{DynamicImage, ImageFormat};
use pdf_writer::{Pdf, Rect, Mm, Page, Content, Finish};
use tempfile::NamedTempFile;
use mime_guess::from_ext;
use serde::Serialize;

#[derive(Serialize)]
struct ErrorResponse {
    error: String,
}

async fn convert_files(mut payload: Multipart) -> Result<HttpResponse, Error> {
    let mut files = Vec::new();
    let mut output_format = "pdf".to_string();

    while let Some(item) = payload.next().await {
        let mut field = item?;
        let content_disposition = field.content_disposition();

        if let Some(name) = content_disposition.get_name() {
            if name == "output_format" {
                let mut format_str = String::new();
                while let Some(chunk) = field.next().await {
                    format_str.push_str(&String::from_utf8_lossy(&chunk?));
                }
                output_format = format_str.to_lowercase();
                continue;
            }
        }

        if let Some(filename) = content_disposition.get_filename() {
            let ext = Path::new(filename)
                .extension()
                .and_then(|s| s.to_str())
                .unwrap_or("")
                .to_lowercase();

            if !["jpg", "jpeg", "png", "hiec"].contains(&ext.as_str()) {
                continue;
            }

            let temp_file = NamedTempFile::new()?;
            let mut file = std::fs::File::create(temp_file.path())?;
            
            while let Some(chunk) = field.next().await {
                let data = chunk?;
                file.write_all(&data)?;
            }

            files.push((temp_file, ext));
        }
    }

    if files.is_empty() {
        return Ok(HttpResponse::BadRequest().json(ErrorResponse {
            error: "No valid files provided".into(),
        }));
    }

    match output_format.as_str() {
        "pdf" => {
            let pdf_bytes = images_to_pdf(&files)?;
            Ok(HttpResponse::Ok()
                .content_type("application/pdf")
                .append_header(ContentDisposition::attachment("converted.pdf"))
                .body(pdf_bytes))
        }
        "jpg" | "jpeg" | "png" => {
            if files.len() > 1 {
                return Ok(HttpResponse::BadRequest().json(ErrorResponse {
                    error: "Can only convert one file at a time to JPG/PNG".into(),
                }));
            }
            let (file, ext) = &files[0];
            let img_bytes = convert_to_image(file.path(), &ext, &output_format)?;
            Ok(HttpResponse::Ok()
                .content_type(from_ext(&output_format).first_or_octet_stream())
                .append_header(ContentDisposition::attachment(format!("converted.{}", output_format)))
                .body(img_bytes))
        }
        _ => Ok(HttpResponse::BadRequest().json(ErrorResponse {
            error: "Unsupported output format".into(),
        })),
    }
}

fn convert_to_image(input_path: &Path, input_ext: &str, output_format: &str) -> io::Result<Vec<u8>> {
    let img = image::open(input_path).map_err(|e| io::Error::new(io::ErrorKind::Other, e))?;
    
    let mut output_bytes = Vec::new();
    let format = match output_format {
        "jpg" | "jpeg" => ImageFormat::Jpeg,
        "png" => ImageFormat::Png,
        _ => return Err(io::Error::new(io::ErrorKind::InvalidInput, "Unsupported format")),
    };
    
    img.write_to(&mut Cursor::new(&mut output_bytes), format)
        .map_err(|e| io::Error::new(io::ErrorKind::Other, e))?;
    Ok(output_bytes)
}

fn images_to_pdf(files: &[(NamedTempFile, String)]) -> io::Result<Vec<u8>> {
    let mut pdf = Pdf::new();
    
    for (file, _) in files {
        let img = image::open(file.path()).map_err(|e| io::Error::new(io::ErrorKind::Other, e))?;
        let (width, height) = img.dimensions();
        
        let width_mm = Mm(width as f32 * 25.4 / 300.0);
        let height_mm = Mm(height as f32 * 25.4 / 300.0);
        
        let mut page = Page::new(Rect::new(Mm(0.0), Mm(0.0), width_mm, height_mm));
        let mut content = Content::new();
        
        content.save_state();
        content.transform(width_mm.0, 0.0, 0.0, height_mm.0, 0.0, 0.0);
        content.text("Image would be embedded here");
        content.restore_state();
        
        page.contents = content.finish();
        pdf.add_page(page);
    }
    
    let mut output = Vec::new();
    pdf.write_to(&mut output)?;
    Ok(output)
}

async fn health_check() -> impl Responder {
    HttpResponse::Ok().json("OK")
}

#[actix_web::main]
async fn main() -> io::Result<()> {
    env_logger::init_from_env(env_logger::Env::new().default_filter_or("info"));

    let port = std::env::var("PORT").unwrap_or_else(|_| "8080".to_string());
    let bind_address = format!("0.0.0.0:{}", port);

    HttpServer::new(|| {
        App::new()
            .wrap(Logger::default())
            .route("/health", web::get().to(health_check))
            .route("/convert", web::post().to(convert_files))
    })
    .bind(&bind_address)?
    .run()
    .await
}
