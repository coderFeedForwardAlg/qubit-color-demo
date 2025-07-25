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
      // Using a relative URL or window.location.hostname to ensure it works in the browser
      // The port is 8081 as defined in the docker-compose.yaml and exposed to the host
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
  
  // Handle file selection
  const handleFileSelect = (event) => {
    const file = event.target.files[0]
    if (file) {
      if (file.type === 'video/mp4') {
        setSelectedFile(file)
        setUploadStatus('File selected: ' + file.name)
        setError(null)
      } else {
        setSelectedFile(null)
        setError('Please select an MP4 file')
        setUploadStatus(null)
      }
    }
  }
  
  // Handle file upload to MinIO via backend
  const uploadToMinIO = async () => {
    if (!selectedFile) {
      setError('Please select a file first')
      return
    }
    
    setLoading(true)
    setUploadStatus('Uploading...')
    setError(null)
    
    try {
      const formData = new FormData()
      formData.append('file', selectedFile)
      
      const response = await fetch('http://localhost:8081/upload-video', {
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
  
  // Trigger file input click
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
      <h1>Vite + React</h1>
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
        
        {/* File selection button */}
        <button 
          onClick={triggerFileInput} 
          disabled={loading}
          style={{ marginRight: '10px' }}
        >
          Select MP4 File
        </button>
        
        {/* Upload to MinIO button */}
        <button 
          onClick={uploadToMinIO} 
          disabled={!selectedFile || loading}
        >
          Upload to MinIO
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
