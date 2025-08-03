from flask import Flask, request, send_file
from flask_cors import CORS
import os, io, zipfile, threading, warnings
from PIL import Image, UnidentifiedImageError, ImageFile
from pillow_heif import register_heif_opener  # <-- Add this line
import fitz  # PyMuPDF
from werkzeug.utils import secure_filename
from pdf2docx import Converter
from docx2pdf import convert as docx_to_pdf

# Enable HEIC support in PIL
register_heif_opener()

# Allow loading large images
ImageFile.LOAD_TRUNCATED_IMAGES = True
warnings.simplefilter('ignore', Image.DecompressionBombWarning)

app = Flask(__name__)
CORS(app)

UPLOAD_FOLDER = "uploads"
CONVERTED_FOLDER = "converted"
os.makedirs(UPLOAD_FOLDER, exist_ok=True)
os.makedirs(CONVERTED_FOLDER, exist_ok=True)

def clear_folder(path):
    for file in os.listdir(path):
        os.remove(os.path.join(path, file))

def convert_file(conversion_type, filepath, name, ext):
    try:
        if conversion_type == "heic_to_jpg" and ext == ".heic":
            img = Image.open(filepath)
            img = img.convert("RGB")
            output_path = os.path.join(CONVERTED_FOLDER, f"{name}.jpg")
            img.save(output_path, "JPEG")

        elif conversion_type == "image_to_pdf" and ext in [".jpg", ".jpeg", ".png"]:
            img = Image.open(filepath).convert("RGB")
            output_path = os.path.join(CONVERTED_FOLDER, f"{name}.pdf")
            img.save(output_path, "PDF")

        elif conversion_type == "pdf_to_jpg" and ext == ".pdf":
            doc = fitz.open(filepath)
            for page_number in range(len(doc)):
                pix = doc.load_page(page_number).get_pixmap()
                output_path = os.path.join(CONVERTED_FOLDER, f"{name}_page{page_number+1}.jpg")
                pix.save(output_path)

        elif conversion_type == "pdf_to_word" and ext == ".pdf":
            output_path = os.path.join(CONVERTED_FOLDER, f"{name}.docx")
            converter = Converter(filepath)
            converter.convert(output_path)
            converter.close()

        elif conversion_type == "word_to_pdf" and ext == ".docx":
            output_path = os.path.join(CONVERTED_FOLDER, f"{name}.pdf")
            docx_to_pdf(filepath, output_path)

        else:
            print(f"[SKIPPED] Unsupported file: {filepath}")

    except (UnidentifiedImageError, Exception) as e:
        print(f"[ERROR] {filepath}: {e}")

@app.route('/convert', methods=['POST'])
def convert_files():
    clear_folder(UPLOAD_FOLDER)
    clear_folder(CONVERTED_FOLDER)

    conversion_type = request.form.get("conversion_type")
    files = request.files.getlist("files")

    threads = []
    for file in files:
        filename = secure_filename(file.filename)
        filepath = os.path.join(UPLOAD_FOLDER, filename)
        file.save(filepath)

        name, ext = os.path.splitext(filename.lower())

        thread = threading.Thread(target=convert_file, args=(conversion_type, filepath, name, ext))
        thread.start()
        threads.append(thread)

    for t in threads:
        t.join()

    zip_buffer = io.BytesIO()
    with zipfile.ZipFile(zip_buffer, "w", zipfile.ZIP_DEFLATED) as zipf:
        for filename in os.listdir(CONVERTED_FOLDER):
            zipf.write(os.path.join(CONVERTED_FOLDER, filename), filename)

    zip_buffer.seek(0)
    return send_file(
        zip_buffer,
        mimetype='application/zip',
        as_attachment=True,
        download_name='converted_files.zip'
    )

if __name__ == '__main__':
    app.run(debug=True, threaded=True)
