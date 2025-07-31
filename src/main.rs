// src/main.rs

use actix_web::{web, App, HttpServer, HttpResponse, Responder};
use actix_multipart::Multipart;
use futures_util::TryStreamExt;
use std::fs::create_dir_all;
use std::io::{BufWriter, Cursor, Write};

use uuid::Uuid;
extern crate image as image_crate;
use image_crate::load_from_memory;

use printpdf::{PdfDocument, Image as PdfImage, ImageTransform, Mm};
use env_logger;

async fn convert_images_to_pdf(mut payload: Multipart) -> impl Responder {
    // 1. Collect all uploaded images in memory
    let mut images = Vec::new();
    while let Ok(Some(mut field)) = payload.try_next().await {
        let mut buf = Vec::new();
        while let Some(chunk) = field.try_next().await.unwrap() {
            buf.extend_from_slice(&chunk);
        }
        if let Ok(img) = load_from_memory(&buf) {
            images.push(img);
        }
    }

    if images.is_empty() {
        return HttpResponse::BadRequest().body("No valid images provided");
    }

    // 2. Create a new A4 PDF
    let (doc, page1, layer1) = PdfDocument::new("Converted PDF", Mm(210.0), Mm(297.0), "Layer 1");
    let layer = doc.get_page(page1).get_layer(layer1);

    // 3. Constants for fitting
    let max_w_mm = 180.0;
    let max_h_mm = 260.0;
    let margin_mm = 10.0;

    // 4. Embed each image
    for img in images {
        let dyn_img = img.to_rgb8().into(); // DynamicImage

        // Convert pixel dims → mm @96dpi
        let (px_w, px_h) = (dyn_img.width(), dyn_img.height());
        let w_mm = px_w as f64 * 0.264583;
        let h_mm = px_h as f64 * 0.264583;
        let scale = (max_w_mm / w_mm).min(max_h_mm / h_mm).min(1.0);

        // High-level helper now exists in 0.8.x:
        let pdf_img = PdfImage::from_dynamic_image(&dyn_img);

        pdf_img.add_to_layer(
            layer.clone(),
            ImageTransform {
                translate_x: Some(Mm(margin_mm)),
                translate_y: Some(Mm(margin_mm)),
                rotate: None,
                // **IMPORTANT**: scale_x/scale_y are raw f64 factors here
                scale_x: Some(scale),
                scale_y: Some(scale),
                dpi: Some(96.0),
            },
        );
    }

    // 5. Write out PDF bytes
    let mut out = Cursor::new(Vec::new());
    doc.save(&mut BufWriter::new(&mut out)).unwrap();
    let pdf_bytes = out.into_inner();

    HttpResponse::Ok()
        .content_type("application/pdf")
        .body(pdf_bytes)
}

// ... rest of your server setup (convert_heic_to_jpg, main, etc.) is unchanged.
