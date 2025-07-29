import React from 'react';
import { FileImage, FileText } from 'lucide-react';
import ConvertHeicToJpg from './components/ConvertHeicToJpg';
import ConvertImageToPdf from './components/ConvertImageToPdf';

function App() {
  return (
    <div className="min-h-screen bg-gradient-to-br from-slate-50 to-blue-50">
      <div className="container mx-auto px-4 py-8">
        {/* Header */}
        <div className="text-center mb-12">
          <div className="flex items-center justify-center gap-3 mb-4">
            <div className="p-3 bg-blue-600 rounded-xl">
              <FileImage className="h-8 w-8 text-white" />
            </div>
            <h1 className="text-4xl font-bold text-gray-900">File Converter</h1>
          </div>
          <p className="text-lg text-gray-600 max-w-2xl mx-auto">
            Convert your files quickly and easily. Transform HEIC images to JPG or combine multiple images into a single PDF.
          </p>
        </div>

        {/* Converters Grid */}
        <div className="grid lg:grid-cols-2 gap-8 max-w-7xl mx-auto">
          <ConvertHeicToJpg />
          <ConvertImageToPdf />
        </div>

        {/* Footer */}
        <div className="text-center mt-16 pt-8 border-t border-gray-200">
          <p className="text-gray-500">
            Secure file conversion powered by advanced algorithms
          </p>
        </div>
      </div>
    </div>
  );
}

export default App;