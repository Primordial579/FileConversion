# 🧰 File Converter (Localhost)

A fast and reliable local file converter that supports up to 1500 files in one go. Easily convert HEIC, JPG, PNG, PDF, and Word formats — all through a modern browser interface. The project is in local host to support more than 1000 file conversion

---

## 🚀 Features

| Conversion Type          | Supported |
|--------------------------|-----------|
| HEIC → JPG               | ✅         |
| JPG/PNG → PDF            | ✅         |
| PDF → JPG                | ✅         |
| PDF → Word (.docx)       | ✅         |
| Word (.docx) → PDF       | ✅         |
| Bulk Conversion (1500+)  | ✅         |
| Multithreading Support   | ✅         |
| Stylish UI               | ✅         |
| Returns ZIP File         | ✅         |

---

## 🛠️ Requirements

- Python 3.7 or higher

### 📦 Install Required Libraries:

```bash
pip install flask flask-cors pillow pillow-heif pdf2docx docx2pdf PyMuPDF
```

---

## ▶️ Running the App

### 1. Start the Backend

```bash
python FileConversion.py
```

### 2. Open the Frontend

Just double-click to open `FileConversion.html` in your browser.

> ⚠️ No need for a frontend server — it's a static HTML file.

---

## 📁 Output

After conversion, a **ZIP file** containing all your converted files will be automatically downloaded.

---

## 🧩 Notes

- HEIC support requires the `pillow-heif` plugin (already listed above).
- File previews are disabled in the frontend to maintain performance for large batches.
- Fully runs on **localhost**. No external server, API, or cloud usage involved.
- More than 200 file conversion takes heavy hardware toll



