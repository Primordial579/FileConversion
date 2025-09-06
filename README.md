# 🧰 File Converter desktop app

A fast and reliable local file converter desktop app that supports up to 1500 files in one go. Easily convert HEIC, JPG, PNG, PDF, and Word formats — all through a modern browser interface.

📥 Download
https://github.com/Primordial579/FileConversion/releases/latest/download/FileConverter.exe


When you download and run the .exe for the first time, Windows may show a warning such as:

“Windows protected your PC” or “This file may contain a virus”

This is normal behavior for custom or unsigned applications.
Since the executable is not signed with a verified publisher certificate, Windows treats it as unrecognized software.

👉 To run the app safely:

1.Click More info.
2.Select Run anyway.


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



