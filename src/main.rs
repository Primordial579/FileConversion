use std::path::{Path, PathBuf};
use std::io::{self, Cursor};
use std::fs::File;
use std::collections::HashMap;

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
use thiserror::Error;
use serde::{Serialize, Deserialize};

#[derive(Error, Debug)]
pub enum ConversionError {
    #[error("IO error: {0}")]
    Io(#[from] io::Error),
    #[error("Image processing error: {0}")]
    Image(#[from] image::ImageError),
    #[error("PDF generation error")]
    PdfError,
    #[error("Unsupported file format")]
    UnsupportedFormat,
    #[error("No files provided")]
    NoFiles,
}

#[derive(Serialize, Deserialize)]
struct ConversionResponse {
    message: String,
    file_url: Option<String>,
}

#[derive(Serialize, Deserialize)]
struct ErrorResponse {
    error: String,
}

// Supported image formats
const SUPPORTED_IMAGE_EXTENSIONS: [&str; 4] = ["jpg", "jpeg", "png", "hiec"];

async fn handle_upload(mut payload: Multipart) -> Result<HttpResponse, Error> {
    let mut files = Vec::new();
    let mut output_format = None;

    while let Some(item) = payload.next().await {
        let mut field = item?;
        let content_disposition = field.content_disposition();

        if let Some(name) = content_disposition.get_name() {
            if name == "output_format" {
                let mut format_str = String::new();
                while let Some(chunk) = field.next().await {
                    format_str.push_str(&String::from_utf8_lossy(&chunk?));
                }
                output_format = Some(format_str);
                continue;
            }
        }

        let filename = content_disposition.get_filename().map(|s| s.to_string());
        if let Some(filename) = filename {
            let ext = Path::new(&filename)
                .extension()
                .and_then(|s| s.to_str())
                .map(|s| s.to_lowercase());

            if let Some(ext) = ext {
                if !SUPPORTED_IMAGE_EXTENSIONS.contains(&ext.as_str()) {
                    continue;
                }
            }

            let temp_file = NamedTempFile::new()?;
            let path = temp_file.path().to_owned();
            
            let mut file = File::create(&path)?;
            while let Some(chunk) = field.next().await {
                let data = chunk?;
                file.write_all(&data)?;
            }

            files.push((path, ext));
        }
    }

    if files.is_empty() {
        return Ok(HttpResponse::BadRequest().json(ErrorResponse {
            error: "No valid files provided".to_string(),
        }));
    }

    let output_format = output_format.as_deref().unwrap_or("pdf");
    match output_format.to_lowercase().as_str() {
        "pdf" => {
            if files.iter().any(|(_, ext)| ext.as_ref().map_or(false, |e| e == "hiec")) {
                return Ok(HttpResponse::BadRequest().json(ErrorResponse {
                    error: "Cannot convert HIEC directly to PDF. Convert to JPG/PNG first.".to_string(),
                }));
            }
            let pdf_bytes = images_to_pdf(files)?;
            Ok(HttpResponse::Ok()
                .content_type("application/pdf")
                .append_header(ContentDisposition::attachment("converted.pdf"))
                .body(pdf_bytes))
        }
        "jpg" | "jpeg" | "png" => {
            if files.len() > 1 {
                return Ok(HttpResponse::BadRequest().json(ErrorResponse {
                    error: "Can only convert one HIEC file at a time to JPG/PNG".to_string(),
                }));
            }
            let (input_path, ext) = &files[0];
            if ext.as_ref().map_or(true, |e| e != "hiec") {
                return Ok(HttpResponse::BadRequest().json(ErrorResponse {
                    error: "Only HIEC files can be converted to JPG/PNG".to_string(),
                }));
            }
            let img_bytes = convert_hiec_to_image(input_path, output_format)?;
            Ok(HttpResponse::Ok()
                .content_type(mime_guess::from_ext(output_format).first_or_octet_stream())
                .append_header(ContentDisposition::attachment(format!("converted.{}", output_format)))
                .body(img_bytes))
        }
        _ => Ok(HttpResponse::BadRequest().json(ErrorResponse {
            error: "Unsupported output format".to_string(),
        })),
    }
}

fn convert_hiec_to_image(input_path: &Path, output_format: &str) -> Result<Vec<u8>, ConversionError> {
    // In a real implementation, you would replace this with actual HIEC decoding
    // For now, we'll assume HIEC is similar to JPEG for demonstration
    let img = image::open(input_path)?;
    
    let mut output_bytes = Vec::new();
    let format = match output_format {
        "jpg" | "jpeg" => ImageFormat::Jpeg,
        "png" => ImageFormat::Png,
        _ => return Err(ConversionError::UnsupportedFormat),
    };
    
    img.write_to(&mut Cursor::new(&mut output_bytes), format)?;
    Ok(output_bytes)
}

fn images_to_pdf(files: Vec<(PathBuf, Option<String>)>) -> Result<Vec<u8>, ConversionError> {
    let mut pdf = Pdf::new();
    
    for (path, _) in files {
        let img = image::open(&path)?;
        let (width, height) = img.dimensions();
        
        // Convert pixels to millimeters (assuming 300 DPI)
        let width_mm = Mm(width as f32 * 25.4 / 300.0);
        let height_mm = Mm(height as f32 * 25.4 / 300.0);
        
        let mut page = Page::new(Rect::new(Mm(0.0), Mm(0.0), width_mm, height_mm));
        let mut content = Content::new();
        
        // Add image to PDF
        content.save_state();
        content.transform(width_mm.0, 0.0, 0.0, height_mm.0, 0.0, 0.0);
        
        // This is a simplified version - in a real implementation you'd need proper PDF image embedding
        content.text("Image would be embedded here");
        
        content.restore_state();
        page.contents = content.finish();
        pdf.add_page(page);
    }
    
    let mut output = Vec::new();
    pdf.finish().map_err(|_| ConversionError::PdfError)?;
    pdf.write_to(&mut output)?;
    
    Ok(output)
}

async fn health_check() -> impl Responder {
    HttpResponse::Ok().json(ConversionResponse {
        message: "Service is healthy".to_string(),
        file_url: None,
    })
}

#[actix_web::main]
async fn main() -> io::Result<()> {
    env_logger::init_from_env(env_logger::Env::new().default_filter_or("info"));

    let port = std::env::var("PORT")
        .unwrap_or_else(|_| "8080".to_string())
        .parse()
        .expect("PORT must be a number");

    HttpServer::new(|| {
        App::new()
            .wrap(Logger::default())
            .route("/health", web::get().to(health_check))
            .route("/convert", web::post().to(handle_upload))
    })
    .bind(("0.0.0.0", port))?
    .run()
    .await
}
