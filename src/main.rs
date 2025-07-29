use actix_web::{web, App, HttpServer, HttpResponse, Responder};
use actix_multipart::Multipart;
use futures_util::stream::StreamExt as _;
use std::fs::File;
use std::io::Write;
use uuid::Uuid;
use image::{DynamicImage, ImageOutputFormat};
use printpdf::*;
use env_logger;

/// Converts HEIC to JPG (simulated – real conversion requires native bindings)
async fn convert_heic_to_jpg(mut payload: Multipart) -> impl Responder {
    while let Ok(Some(mut field)) = payload.try_next().await {
        let filename = Uuid::new_v4().to_string() + ".heic";
        let filepath = format!("./tmp/{}", filename);

        let mut f = File::create(&filepath).unwrap();
        while let Some(chunk) = field.next().await {
            f.write_all(&chunk.unwrap()).unwrap();
        }

        println!("Received and saved HEIC file: {}", filename);

        // Simulate conversion (actual HEIC decoding needs external tools or C bindings)
        return HttpResponse::Ok().body("HEIC conversion simulated (real support requires native bindings)");
    }

    HttpResponse::BadRequest().body("No file uploaded")
}

/// Combines JPG/PNG images into a single PDF
async fn convert_images_to_pdf(mut payload: Multipart) -> impl Responder {
    let mut images: Vec<DynamicImage> = Vec::new();

    while let Ok(Some(mut field)) = payload.try_next().await {
        let mut data = vec![];
        while let Some(chunk) = field.next().await {
            data.extend_from_slice(&chunk.unwrap());
        }

        if let Ok(img) = image::load_from_memory(&data) {
            images.push(img);
        }
    }

    if images.is_empty() {
        return HttpResponse::BadRequest().body("No valid images provided");
    }

    println!("Processing {} image(s) to PDF...", images.len());

    let (doc, page1, layer1) = PdfDocument::new("Converted PDF", Mm(210.0), Mm(297.0), "Layer 1");
    let current_layer = doc.get_page(page1).get_layer(layer1);

    // Placeholder text (since image drawing in PDF is not implemented here)
    current_layer.use_text(
        "PDF created (actual image placement not implemented)",
        12.0,
        Mm(10.0),
        Mm(280.0),
        &doc.get_font("Helvetica").unwrap(),
    );

    let mut buffer = Vec::new();
    doc.save(&mut buffer).unwrap();

    HttpResponse::Ok()
        .content_type("application/pdf")
        .body(buffer)
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    std::fs::create_dir_all("./tmp").unwrap();

    env_logger::init(); // Optional logger

    println!("🚀 Starting server on 0.0.0.0:8080");

    HttpServer::new(|| {
        App::new()
            .route("/convert/heic-to-jpg", web::post().to(convert_heic_to_jpg))
            .route("/convert/image-to-pdf", web::post().to(convert_images_to_pdf))
    })
    .bind("0.0.0.0:8080")?  // ✅ Required for Render
    .run()
    .await
}
