<script lang="ts">
  import './app.css';
  import Layout from './lib/Layout.svelte';
  import { onMount } from 'svelte';
  import { log } from './lib/logger.js';
  
  let currentFilePath: string | null = null;
  let isInTauri = false;
  
  // Handle keyboard shortcuts
  function handleKeydown(event: KeyboardEvent) {
    log.debug(`Key pressed: ${event.key}, Ctrl: ${event.ctrlKey}, Meta: ${event.metaKey}`);
    
    // Support both Ctrl (Windows/Linux) and Cmd (macOS)
    const isModifierPressed = event.ctrlKey || event.metaKey;
    
    // Ctrl+O or Cmd+O: Open file
    if (isModifierPressed && event.key === 'o') {
      event.preventDefault();
      log.info('Ctrl+O/Cmd+O detected - Opening file dialog');
      openFile();
    }
    
    // Ctrl+P or Cmd+P: Process current file with Docling
    if (isModifierPressed && event.key === 'p') {
      event.preventDefault();
      log.info('Ctrl+P/Cmd+P detected - Processing file');
      if (currentFilePath) {
        processFile(currentFilePath);
      } else {
        log.info('No file selected - prompting user');
        alert('No file selected. Use Ctrl+O to open a file first.');
      }
    }
  }
  
  async function openFile() {
    try {
      if (isInTauri) {
        const { invoke } = await import('@tauri-apps/api/core');
        const result = await invoke('select_pdf_file');
        if (result?.success && result?.path) {
          currentFilePath = result.path;
          log.info(`File opened: ${result.path}`);
        }
      } else {
        log.info('File selection only works in Tauri app, not browser');
        alert('🐹 File selection only works in Tauri app, not browser');
      }
    } catch (error) {
      log.error(`Failed to open file: ${error}`);
      alert('Failed to open file: ' + error);
    }
  }
  
  async function processFile(filePath: string) {
    try {
      log.info(`Processing file with Docling: ${filePath}`);
      if (isInTauri) {
        const { invoke } = await import('@tauri-apps/api/core');
        const result = await invoke('process_with_docling', { filePath });
        if (result?.success) {
          log.info(`Docling result: ${result.message}`);
          alert('✅ Docling processing completed!\n\n' + result.message);
        }
      } else {
        log.info('Docling processing only works in Tauri app, not browser');
        alert('🐹 Docling processing only works in Tauri app, not browser');
      }
    } catch (error) {
      log.error(`Failed to process file: ${error}`);
      alert('Failed to process file: ' + error);
    }
  }
  
  onMount(() => {
    // Check if we're running in Tauri
    isInTauri = '__TAURI__' in window;
    log.info(`Running in Tauri: ${isInTauri}`);
    
    // Add keyboard event listener with multiple approaches
    log.info('Adding keyboard event listeners...');
    
    const keydownHandler = (event: KeyboardEvent) => {
      log.debug(`KEYDOWN EVENT CAPTURED: key=${event.key}, code=${event.code}, ctrl=${event.ctrlKey}, meta=${event.metaKey}`);
      handleKeydown(event);
    };
    
    // Try multiple event listeners
    window.addEventListener('keydown', keydownHandler, true); // capture phase
    document.addEventListener('keydown', keydownHandler, true); // capture phase
    document.body.addEventListener('keydown', keydownHandler, true); // capture phase
    
    return () => {
      window.removeEventListener('keydown', keydownHandler, true);
      document.removeEventListener('keydown', keydownHandler, true);
      document.body.removeEventListener('keydown', keydownHandler, true);
    };
  });
</script>

<div class="h-screen bg-term-bg text-term-fg font-mono flex flex-col">
  <!-- Simple header -->
  <div class="flex justify-between items-center px-4 py-1 text-xs text-term-dim border-b border-term-border">
    <span class="text-term-bright">🐹 CHONKER - PDF Preprocessor</span>
    <div class="flex gap-4">
      <span>Cmd+O: Open</span>
      <span>Cmd+P: Process</span>
      <span class="text-term-bright">READY</span>
    </div>
  </div>
  
  <!-- Test buttons for debugging -->
  <div class="p-4 border-b border-term-border">
    <div class="flex gap-4">
      <button 
        class="px-4 py-2 bg-term-bright text-term-bg rounded hover:bg-term-dim"
        on:click={() => openFile()}
      >
        🐹 Test Open File
      </button>
      <button 
        class="px-4 py-2 bg-term-bright text-term-bg rounded hover:bg-term-dim"
        on:click={() => currentFilePath ? processFile(currentFilePath) : alert('No file selected')}
      >
        🐹 Test Process File
      </button>
      <span class="text-term-dim">Current file: {currentFilePath || 'None'}</span>
    </div>
  </div>
  
  <!-- Main content -->
  <div class="flex-1 flex">
    <Layout />
  </div>
</div>
