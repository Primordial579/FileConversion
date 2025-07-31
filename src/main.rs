use actix_web::{web, App, HttpServer, HttpResponse, Responder};
use actix_multipart::Multipart;
use futures_util::stream::StreamExt as _;
use std::fs::{File, create_dir_all};
use std::io::{Write, BufWriter};
use uuid::Uuid;
use image::{DynamicImage, ImageOutputFormat, GenericImageView};
use printpdf::*;
use std::io::Cursor;
use env_logger;

/// Converts HEIC to JPG (simulated)
async fn convert_heic_to_jpg(mut payload: Multipart) -> impl Responder {
    while let Ok(Some(mut field)) = payload.try_next().await {
        let filename = Uuid::new_v4().to_string() + ".heic";
        let filepath = format!("./tmp/{}", filename);

        let mut f = File::create(&filepath).unwrap();
        while let Some(chunk) = field.next().await {
            f.write_all(&chunk.unwrap()).unwrap();
        }

        println!("Received and saved HEIC file: {}", filename);

        return HttpResponse::Ok().body("HEIC conversion simulated (real support requires native bindings)");
    }

    HttpResponse::BadRequest().body("No file uploaded")
}

/// Converts multiple images (JPG/PNG) into a single PDF
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

    let max_width_mm = 180.0;
    let max_height_mm = 260.0;

    for img in images {
        let rgb = img.to_rgb8();
        let (w, h) = rgb.dimensions();
        let image = Image::from_dynamic_image(&DynamicImage::ImageRgb8(rgb));

        let width_mm = w as f64 * 0.264583;
        let height_mm = h as f64 * 0.264583;

        let scale_x = max_width_mm / width_mm;
        let scale_y = max_height_mm / height_mm;
        let scale = scale_x.min(scale_y).min(1.0);  // keep image within page

        image.add_to_layer(
            current_layer.clone(),
            ImageTransform {
                translate_x: Some(Mm(10.0)),
                translate_y: Some(Mm(10.0)),
                scale_x: Some(Mm(width_mm * scale)),
                scale_y: Some(Mm(height_mm * scale)),
                rotate: None,
                dpi: Some(96.0),
            },
        );
    }

    let mut buffer = Cursor::new(Vec::new());
    doc.save(&mut BufWriter::new(&mut buffer)).unwrap();
    let pdf_data = buffer.into_inner();

    HttpResponse::Ok()
        .content_type("application/pdf")
        .body(pdf_data)
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    create_dir_all("./tmp").unwrap();
    env_logger::init();
    println!("🚀 Starting server on 0.0.0.0:8080");

    HttpServer::new(|| {
        App::new()
            .route("/convert/heic-to-jpg", web::post().to(convert_heic_to_jpg))
            .route("/convert/image-to-pdf", web::post().to(convert_images_to_pdf))
    })
    .bind("0.0.0.0:8080")?
    .run()
    .await
}
