import React, { useState, useRef } from 'react';
import { Upload, Download, Loader2, CheckCircle, AlertCircle, Archive } from 'lucide-react';

interface ConvertedFile {
  originalName: string;
  downloadUrl: string;
}

const ConvertHeicToJpg: React.FC = () => {
  const [files, setFiles] = useState<File[]>([]);
  const [isLoading, setIsLoading] = useState(false);
  const [convertedFiles, setConvertedFiles] = useState<ConvertedFile[]>([]);
  const [error, setError] = useState<string>('');
  const [isDragOver, setIsDragOver] = useState(false);
  const fileInputRef = useRef<HTMLInputElement>(null);

  const handleDragOver = (e: React.DragEvent) => {
    e.preventDefault();
    setIsDragOver(true);
  };

  const handleDragLeave = (e: React.DragEvent) => {
    e.preventDefault();
    setIsDragOver(false);
  };

  const handleDrop = (e: React.DragEvent) => {
    e.preventDefault();
    setIsDragOver(false);
    const droppedFiles = Array.from(e.dataTransfer.files);
    const heicFiles = droppedFiles.filter(file => 
      file.type === 'image/heic' || file.name.toLowerCase().endsWith('.heic')
    );
    setFiles(heicFiles);
    setError('');
    setConvertedFiles([]);
  };

  const handleFileSelect = (e: React.ChangeEvent<HTMLInputElement>) => {
    if (e.target.files) {
      const selectedFiles = Array.from(e.target.files);
      setFiles(selectedFiles);
      setError('');
      setConvertedFiles([]);
    }
  };

  const handleConvert = async () => {
    if (files.length === 0) {
      setError('Please select at least one HEIC file');
      return;
    }

    setIsLoading(true);
    setError('');
    setConvertedFiles([]);

    try {
      const formData = new FormData();
      files.forEach(file => {
        formData.append('files', file);
      });

      const response = await fetch('https://fileconversion-s262.onrender.com/convert/heic-to-jpg', {
        method: 'POST',
        body: formData,
      });

      if (!response.ok) {
        throw new Error(`Conversion failed: ${response.statusText}`);
      }

      // Assuming the API returns JSON with download URLs
      const result = await response.json();
      
      // Handle different possible response formats
      if (Array.isArray(result)) {
        setConvertedFiles(result.map((item: any, index: number) => ({
          originalName: files[index]?.name || `file-${index}`,
          downloadUrl: item.downloadUrl || item.url || item
        })));
      } else if (result.files) {
        setConvertedFiles(result.files);
      } else {
        // If response is a blob/file, handle as single download
        const blob = await response.blob();
        const url = URL.createObjectURL(blob);
        setConvertedFiles([{
          originalName: files[0]?.name || 'converted',
          downloadUrl: url
        }]);
      }
    } catch (err) {
      setError(err instanceof Error ? err.message : 'Conversion failed');
    } finally {
      setIsLoading(false);
    }
  };

  const handleDownloadAll = () => {
    // Create a simple zip-like download by triggering all downloads
    convertedFiles.forEach((file, index) => {
      setTimeout(() => {
        const link = document.createElement('a');
        link.href = file.downloadUrl;
        link.download = file.originalName.replace('.heic', '.jpg');
        document.body.appendChild(link);
        link.click();
        document.body.removeChild(link);
      }, index * 100); // Stagger downloads slightly
    });
  };

  const removeFile = (index: number) => {
    setFiles(files.filter((_, i) => i !== index));
  };

  return (
    <div className="bg-white rounded-2xl shadow-lg border border-gray-100 overflow-hidden">
      {/* Header */}
      <div className="bg-gradient-to-r from-blue-500 to-blue-600 px-6 py-4">
        <div className="flex items-center gap-3">
          <Upload className="h-6 w-6 text-white" />
          <h2 className="text-xl font-semibold text-white">HEIC to JPG Converter</h2>
        </div>
        <p className="text-blue-100 mt-1">Convert your HEIC images to JPG format</p>
      </div>

      <div className="p-6">
        {/* Upload Zone */}
        <div
          className={`border-2 border-dashed rounded-xl p-8 text-center transition-all duration-200 ${
            isDragOver
              ? 'border-blue-400 bg-blue-50'
              : 'border-gray-300 hover:border-blue-400 hover:bg-gray-50'
          }`}
          onDragOver={handleDragOver}
          onDragLeave={handleDragLeave}
          onDrop={handleDrop}
          onClick={() => fileInputRef.current?.click()}
        >
          <Upload className="h-12 w-12 text-gray-400 mx-auto mb-4" />
          <p className="text-lg font-medium text-gray-900 mb-2">
            Drop HEIC files here or click to browse
          </p>
          <p className="text-gray-500">Supports multiple .heic files</p>
          <input
            ref={fileInputRef}
            type="file"
            multiple
            accept=".heic,image/heic"
            onChange={handleFileSelect}
            className="hidden"
          />
        </div>

        {/* Selected Files */}
        {files.length > 0 && (
          <div className="mt-6">
            <h3 className="font-medium text-gray-900 mb-3">Selected Files ({files.length})</h3>
            <div className="space-y-2 max-h-40 overflow-y-auto">
              {files.map((file, index) => (
                <div key={index} className="flex items-center justify-between bg-gray-50 p-3 rounded-lg">
                  <span className="text-sm text-gray-700 truncate">{file.name}</span>
                  <button
                    onClick={() => removeFile(index)}
                    className="text-red-500 hover:text-red-700 text-sm font-medium"
                  >
                    Remove
                  </button>
                </div>
              ))}
            </div>
          </div>
        )}

        {/* Convert Button */}
        <button
          onClick={handleConvert}
          disabled={files.length === 0 || isLoading}
          className="w-full mt-6 bg-blue-600 hover:bg-blue-700 disabled:bg-gray-300 text-white font-medium py-3 px-4 rounded-xl transition-colors duration-200 flex items-center justify-center gap-2"
        >
          {isLoading ? (
            <>
              <Loader2 className="h-5 w-5 animate-spin" />
              Converting...
            </>
          ) : (
            <>
              <CheckCircle className="h-5 w-5" />
              Convert to JPG
            </>
          )}
        </button>

        {/* Error Message */}
        {error && (
          <div className="mt-4 p-4 bg-red-50 border border-red-200 rounded-lg flex items-start gap-3">
            <AlertCircle className="h-5 w-5 text-red-500 flex-shrink-0 mt-0.5" />
            <p className="text-red-700">{error}</p>
          </div>
        )}

        {/* Converted Files */}
        {convertedFiles.length > 0 && (
          <div className="mt-6">
            <div className="flex items-center justify-between mb-4">
              <h3 className="font-medium text-gray-900">Converted Files</h3>
              {convertedFiles.length > 1 && (
                <button
                  onClick={handleDownloadAll}
                  className="flex items-center gap-2 text-blue-600 hover:text-blue-700 font-medium text-sm"
                >
                  <Archive className="h-4 w-4" />
                  Download All
                </button>
              )}
            </div>
            <div className="space-y-2">
              {convertedFiles.map((file, index) => (
                <div key={index} className="flex items-center justify-between bg-green-50 p-3 rounded-lg">
                  <span className="text-sm text-gray-700 truncate">
                    {file.originalName.replace('.heic', '.jpg')}
                  </span>
                  <a
                    href={file.downloadUrl}
                    download={file.originalName.replace('.heic', '.jpg')}
                    className="flex items-center gap-2 bg-green-600 hover:bg-green-700 text-white text-sm font-medium px-3 py-1.5 rounded-lg transition-colors duration-200"
                  >
                    <Download className="h-4 w-4" />
                    Download
                  </a>
                </div>
              ))}
            </div>
          </div>
        )}
      </div>
    </div>
  );
};

export default ConvertHeicToJpg;