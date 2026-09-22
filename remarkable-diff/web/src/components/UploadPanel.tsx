import { useState, useCallback } from 'react'
import { useMutation } from '@tanstack/react-query'
import { X, Upload, File, Loader2 } from 'lucide-react'
import clsx from 'clsx'
import * as api from '../lib/api'

interface Props {
  onClose: () => void
  onSuccess: () => void
}

export default function UploadPanel({ onClose, onSuccess }: Props) {
  const [name, setName] = useState('')
  const [files, setFiles] = useState<File[]>([])
  const [dragOver, setDragOver] = useState(false)
  
  const uploadMutation = useMutation({
    mutationFn: async () => {
      const formData = new FormData()
      formData.append('name', name || 'Untitled')
      files.forEach((file) => {
        if (file.name.endsWith('.rm')) {
          formData.append(file.name, file)
        } else if (file.name === '.content' || file.name.endsWith('.content')) {
          formData.append('content', file)
        }
      })
      return api.uploadDocument(formData)
    },
    onSuccess: () => {
      onSuccess()
    },
  })
  
  const handleDrop = useCallback((e: React.DragEvent) => {
    e.preventDefault()
    setDragOver(false)
    
    const dropped = Array.from(e.dataTransfer.files).filter(
      (f) => f.name.endsWith('.rm') || f.name.endsWith('.content')
    )
    setFiles((prev) => [...prev, ...dropped])
  }, [])
  
  const handleFileInput = useCallback((e: React.ChangeEvent<HTMLInputElement>) => {
    if (e.target.files) {
      const selected = Array.from(e.target.files)
      setFiles((prev) => [...prev, ...selected])
    }
  }, [])
  
  const removeFile = (index: number) => {
    setFiles((prev) => prev.filter((_, i) => i !== index))
  }
  
  return (
    <div className="fixed inset-0 bg-black/50 flex items-center justify-center z-50">
      <div className="bg-gray-900 rounded-xl w-full max-w-lg p-6">
        <div className="flex items-center justify-between mb-6">
          <h2 className="text-xl font-bold">Upload Document</h2>
          <button onClick={onClose} className="p-1 hover:bg-gray-800 rounded">
            <X className="w-5 h-5" />
          </button>
        </div>
        
        {/* Name input */}
        <div className="mb-4">
          <label className="block text-sm text-gray-400 mb-1">Document Name</label>
          <input
            type="text"
            value={name}
            onChange={(e) => setName(e.target.value)}
            placeholder="My Document"
            className="w-full bg-gray-800 rounded-lg px-4 py-2"
          />
        </div>
        
        {/* Drop zone */}
        <div
          onDragOver={(e) => {
            e.preventDefault()
            setDragOver(true)
          }}
          onDragLeave={() => setDragOver(false)}
          onDrop={handleDrop}
          className={clsx(
            'border-2 border-dashed rounded-lg p-8 text-center transition-colors',
            dragOver ? 'border-blue-500 bg-blue-500/10' : 'border-gray-700'
          )}
        >
          <Upload className="w-10 h-10 mx-auto mb-3 text-gray-500" />
          <p className="text-gray-400 mb-2">
            Drag and drop .rm files here, or{' '}
            <label className="text-blue-400 hover:underline cursor-pointer">
              browse
              <input
                type="file"
                multiple
                accept=".rm,.content"
                onChange={handleFileInput}
                className="hidden"
              />
            </label>
          </p>
          <p className="text-sm text-gray-500">
            Supports .rm (lines) and .content files
          </p>
        </div>
        
        {/* File list */}
        {files.length > 0 && (
          <div className="mt-4 space-y-2">
            {files.map((file, index) => (
              <div
                key={`${file.name}-${index}`}
                className="flex items-center gap-2 bg-gray-800 rounded-lg px-3 py-2"
              >
                <File className="w-4 h-4 text-gray-400" />
                <span className="flex-1 truncate text-sm">{file.name}</span>
                <span className="text-xs text-gray-500">
                  {(file.size / 1024).toFixed(1)} KB
                </span>
                <button
                  onClick={() => removeFile(index)}
                  className="p-1 hover:bg-gray-700 rounded"
                >
                  <X className="w-4 h-4" />
                </button>
              </div>
            ))}
          </div>
        )}
        
        {/* Upload button */}
        <div className="mt-6 flex gap-3">
          <button
            onClick={onClose}
            className="flex-1 px-4 py-2 bg-gray-800 hover:bg-gray-700 rounded-lg"
          >
            Cancel
          </button>
          <button
            onClick={() => uploadMutation.mutate()}
            disabled={files.length === 0 || uploadMutation.isPending}
            className="flex-1 px-4 py-2 bg-blue-600 hover:bg-blue-700 rounded-lg disabled:opacity-50 flex items-center justify-center gap-2"
          >
            {uploadMutation.isPending ? (
              <>
                <Loader2 className="w-4 h-4 animate-spin" />
                Uploading...
              </>
            ) : (
              <>
                <Upload className="w-4 h-4" />
                Upload
              </>
            )}
          </button>
        </div>
        
        {uploadMutation.isError && (
          <p className="mt-3 text-red-400 text-sm text-center">
            Upload failed. Please try again.
          </p>
        )}
      </div>
    </div>
  )
}
