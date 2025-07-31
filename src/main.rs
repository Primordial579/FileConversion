// src/main.rs

use actix_web::{web, App, HttpServer, HttpResponse, Responder};
use actix_multipart::Multipart;
use futures_util::TryStreamExt;                    // bring try_next into scope
use std::fs::{create_dir_all, File};
use std::io::{BufWriter, Cursor, Write};
use uuid::Uuid;

// disambiguate the `image` crate
extern crate image as image_crate;
use image_crate::{DynamicImage, GenericImageView, load_from_memory};

use printpdf::{
    PdfDocument, ImageXObject, ImageTransform, Mm, Px, ColorSpace, ColorBits
};
use env_logger;

async fn convert_heic_to_jpg(mut payload: Multipart) -> impl Responder {
    while let Ok(Some(mut field)) = payload.try_next().await {
        let filename = format!("{}.heic", Uuid::new_v4());
        let path = format!("./tmp/{}", filename);
        let mut f = File::create(&path).unwrap();

        while let Some(chunk) = field.try_next().await.unwrap() {
            f.write_all(&chunk).unwrap();
        }

        println!("Saved HEIC: {}", filename);
        return HttpResponse::Ok()
            .body("HEIC conversion simulated (native bindings needed for real)");
    }
    HttpResponse::BadRequest().body("No file uploaded")
}

async fn convert_images_to_pdf(mut payload: Multipart) -> impl Responder {
    let mut images = Vec::<DynamicImage>::new();

    while let Ok(Some(mut field)) = payload.try_next().await {
        let mut data = Vec::new();
        while let Some(chunk) = field.try_next().await.unwrap() {
            data.extend_from_slice(&chunk);
        }
        if let Ok(img) = load_from_memory(&data) {
            images.push(img);
        }
    }

    if images.is_empty() {
        return HttpResponse::BadRequest().body("No valid images");
    }

    println!("Converting {} image(s) → PDF", images.len());

    // A4 portrait
    let (doc, page1, layer1) = PdfDocument::new("Converted", Mm(210.0), Mm(297.0), "L1");
    let layer = doc.get_page(page1).get_layer(layer1);

    // max printable area in mm
    let max_w_mm = 180.0;
    let max_h_mm = 260.0;

    for img in images {
        let rgb8 = img.to_rgb8();
        let (px_w, px_h) = rgb8.dimensions();
        let raw = rgb8.into_raw();

        // convert pixels → mm @96 DPI
        let w_mm = px_w as f64 * 0.264583;
        let h_mm = px_h as f64 * 0.264583;
        let scale = (max_w_mm / w_mm).min(max_h_mm / h_mm).min(1.0);

        // build XObject
        let xobj = ImageXObject::new(
            Px(px_w as usize),
            Px(px_h as usize),
            ColorSpace::Rgb,
            ColorBits::Bit8,
            false,
            None,
            None,
            raw,
        );

        // place it at (10 mm, 10 mm) from bottom-left, scaled
        xobj.add_to_layer(
            layer.clone(),
            ImageTransform {
                translate_x: Some(Mm(10.0)),
                translate_y: Some(Mm(10.0)),
                rotate: None,
                scale_x: Some(scale),
                scale_y: Some(scale),
                dpi: Some(96.0),
            },
        );
    }

    let mut buf = Cursor::new(Vec::new());
    doc.save(&mut BufWriter::new(&mut buf)).unwrap();
    let pdf = buf.into_inner();

    HttpResponse::Ok()
        .content_type("application/pdf")
        .body(pdf)
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    create_dir_all("./tmp")?;
    env_logger::init();
    println!("🚀 Running on 0.0.0.0:8080");
    HttpServer::new(|| {
        App::new()
            .route("/convert/heic-to-jpg", web::post().to(convert_heic_to_jpg))
            .route("/convert/image-to-pdf", web::post().to(convert_images_to_pdf))
    })
    .bind("0.0.0.0:8080")?
    .run()
    .await
}
