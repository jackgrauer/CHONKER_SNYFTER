<script lang="ts">
  import { onMount } from 'svelte';
  import './app.css';
  import Layout from './lib/Layout.svelte';
  import { setupKeyboardShortcuts } from './lib/keyboard';
  
  let time = new Date().toLocaleTimeString('en-US', { hour12: false });
  
  onMount(() => {
    // Update clock
    const interval = setInterval(() => {
      time = new Date().toLocaleTimeString('en-US', { hour12: false });
    }, 1000);
    
    // Setup keyboard shortcuts
    setupKeyboardShortcuts();
    
    return () => clearInterval(interval);
  });
</script>

<div class="h-screen bg-term-bg text-term-fg font-mono flex flex-col">
  <!-- Terminal header bar -->
  <div class="flex justify-between items-center px-4 py-1 text-xs text-term-dim border-b border-term-border">
    <div class="flex items-center space-x-4">
      <span class="text-term-bright">🐹 CHONKER v2.0</span>
      <span class="text-term-dim">Universal Document Processor</span>
    </div>
    <span class="font-mono">{time}</span>
  </div>
  
  <!-- Main content area -->
  <div class="flex-1 flex">
    <Layout />
  </div>
  
  <!-- Terminal status line -->
  <div class="px-4 py-1 text-xs text-term-dim border-t border-term-border flex justify-between">
    <div class="flex space-x-4">
      <span><span class="text-term-bright">[SPACE]</span> Load</span>
      <span><span class="text-term-bright">[Ctrl+P]</span> Process</span>
      <span><span class="text-term-bright">[Ctrl+:]</span> Command</span>
      <span><span class="text-term-bright">[?]</span> Help</span>
    </div>
    <span>READY</span>
  </div>
</div>
