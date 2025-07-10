<script lang="ts">
  import './app.css';
  import Layout from './lib/Layout.svelte';
</script>

<div class="h-screen bg-term-bg text-term-fg font-mono flex flex-col">
  <!-- Simple header -->
  <div class="flex justify-between items-center px-4 py-1 text-xs text-term-dim border-b border-term-border">
    <span class="text-term-bright">🐹 CHONKER - PDF Preprocessor</span>
    <span>READY</span>
  </div>
  
  <!-- Main content -->
  <div class="flex-1 flex">
    <Layout />
  </div>
</div>
