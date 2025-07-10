<script lang="ts">
  import './app.css';
  import Layout from './lib/Layout.svelte';
  import { invoke } from '@tauri-apps/api';
  import { onMount } from 'svelte';
  import { document } from './stores/document';
  
  let currentFilePath: string | null = null;
  
  // Handle keyboard shortcuts
  function handleKeydown(event: KeyboardEvent) {
    // Ctrl+O: Open file
    if (event.ctrlKey && event.key === 'o') {
      event.preventDefault();
      openFile();
    }
    
    // Ctrl+P: Process current file with Docling
    if (event.ctrlKey && event.key === 'p') {
      event.preventDefault();
      if (currentFilePath) {
        processFile(currentFilePath);
      } else {
        alert('No file selected. Use Ctrl+O to open a file first.');
      }
    }
  }
  
  async function openFile() {
    try {
      const result = await invoke('select_pdf_file');
      if (result?.success && result?.path) {
        currentFilePath = result.path;
        document.setPath(result.path);
        console.log('🐹 File opened:', result.path);
      }
    } catch (error) {
      console.error('Failed to open file:', error);
      alert('Failed to open file: ' + error);
    }
  }
  
  async function processFile(filePath: string) {
    try {
      console.log('🐹 Processing file with Docling:', filePath);
      const result = await invoke('process_with_docling', { filePath });
      if (result?.success) {
        console.log('✅ Docling result:', result.message);
        alert('✅ Docling processing completed!\n\n' + result.message);
      }
    } catch (error) {
      console.error('Failed to process file:', error);
      alert('Failed to process file: ' + error);
    }
  }
  
  onMount(() => {
    // Add keyboard event listener
    window.addEventListener('keydown', handleKeydown);
    
    return () => {
      window.removeEventListener('keydown', handleKeydown);
    };
  });
</script>

<div class="h-screen bg-term-bg text-term-fg font-mono flex flex-col">
  <!-- Simple header -->
  <div class="flex justify-between items-center px-4 py-1 text-xs text-term-dim border-b border-term-border">
    <span class="text-term-bright">🐹 CHONKER - PDF Preprocessor</span>
    <div class="flex gap-4">
      <span>Ctrl+O: Open</span>
      <span>Ctrl+P: Process</span>
      <span class="text-term-bright">READY</span>
    </div>
  </div>
  
  <!-- Main content -->
  <div class="flex-1 flex">
    <Layout />
  </div>
</div>
