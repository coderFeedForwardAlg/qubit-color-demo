import { useState, useEffect, useRef } from 'react'
import reactLogo from './assets/react.svg'
import viteLogo from '/vite.svg'
import './App.css'

function App() {
  const [count, setCount] = useState(0)
  const [backendData, setBackendData] = useState(null)
  const [loading, setLoading] = useState(false)
  const [error, setError] = useState(null)
  
  // File upload state
  const [selectedFile, setSelectedFile] = useState(null)
  const [uploadStatus, setUploadStatus] = useState(null)
  const [uploadProgress, setUploadProgress] = useState(0)
  const fileInputRef = useRef(null)

  const fetchFromBackend = async () => {
    setLoading(true)
    setError(null)
    try {
      // Using localhost to access the backend from the browser
      // The port 8081 is exposed in docker-compose.yaml
      const response = await fetch('http://localhost:8081/health')
      
      if (!response.ok) {
        throw new Error(`Backend responded with status: ${response.status}`)
      }
      
      const data = await response.json()
      setBackendData(data)
    } catch (err) {
      console.error('Error fetching from backend:', err)
      setError(err.message)
    } finally {
      setLoading(false)
    }
  }
  
  // Handle file selection and upload
  const handleFileSelect = async (event) => {
    const file = event.target.files[0]
    if (!file) return
    
    if (file.type !== 'video/mp4') {
      setSelectedFile(null)
      setError('Please select an MP4 file')
      setUploadStatus(null)
      return
    }
    
    setSelectedFile(file)
    setUploadStatus('Uploading ' + file.name + '...')
    setError(null)
    
    // Proceed with upload
    const file_name = file.name
    setLoading(true)
    
    try {
      const formData = new FormData()
      formData.append('file', file)
      
      // Sanitize and encode the filename
      const safeFileName = file_name.replace(/[^a-zA-Z0-9\-._]/g, '');
      
      // Use localhost to access the backend from the browser
      const response = await fetch(`http://localhost:8081/upload-raw-video?bucket=bucket&file=${encodeURIComponent(safeFileName)}`, {
        method: 'POST',
        body: formData,
      })
      
      if (!response.ok) {
        throw new Error(`Upload failed with status: ${response.status}`)
      }
      
      const result = await response.json()
      setUploadStatus('Upload successful: ' + result.file_name)
      setBackendData(result)
    } catch (err) {
      console.error('Error uploading file:', err)
      setError(err.message)
      setUploadStatus('Upload failed')
    } finally {
      setLoading(false)
    }
  }
  
  // Function to trigger file input click
  const triggerFileInput = () => {
    fileInputRef.current.click()
  }

  return (
    <>
      <div>
        <a href="https://vite.dev" target="_blank" rel="noreferrer">
          <img src={viteLogo} className="logo" alt="Vite logo" />
        </a>
        <a href="https://react.dev" target="_blank" rel="noreferrer">
          <img src={reactLogo} className="logo react" alt="React logo" />
        </a>
      </div>
      <h1>Vite + React + MinIO</h1>
      <div className="card">
        <button onClick={() => setCount((count) => count + 1)}>
          count is {count}
        </button>
        <p>
          Edit <code>src/App.jsx</code> and save to test HMR
        </p>
      </div>
      
      <div className="card">
        <h2>Backend Connection</h2>
        <button onClick={fetchFromBackend} disabled={loading}>
          {loading ? 'Loading...' : 'Fetch from Backend'}
        </button>
        
        {error && (
          <div className="error" style={{ color: 'red', marginTop: '10px' }}>
            Error: {error}
          </div>
        )}
        
        {backendData && (
          <div className="data" style={{ marginTop: '10px' }}>
            <pre>{JSON.stringify(backendData, null, 2)}</pre>
          </div>
        )}
      </div>
      
      <div className="card">
        <h2>MP4 File Upload</h2>
        
        {/* Hidden file input */}
        <input 
          type="file" 
          ref={fileInputRef} 
          onChange={handleFileSelect} 
          accept="video/mp4" 
          style={{ display: 'none' }} 
        />
        
        {/* Single upload button */}
        <button 
          onClick={triggerFileInput} 
          disabled={loading}
        >
          {loading ? 'Uploading...' : 'Upload MP4 to MinIO'}
        </button>
        
        {/* Upload status */}
        {uploadStatus && (
          <div style={{ marginTop: '10px' }}>
            {uploadStatus}
          </div>
        )}
        
        {/* Error message */}
        {error && (
          <div style={{ color: 'red', marginTop: '10px' }}>
            Error: {error}
          </div>
        )}
      </div>
      
      <p className="read-the-docs">
        Click on the Vite and React logos to learn more
      </p>
    </>
  )
}

export default App
