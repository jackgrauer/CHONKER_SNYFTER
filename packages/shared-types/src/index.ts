// Document Processing Types
export interface DocumentInfo {
  path: string;
  name: string;
  size: number;
  type: string;
  lastModified: Date;
}

export interface ProcessingResult {
  success: boolean;
  message: string;
  file_path: string;
  extracted_text?: string;
  tables?: Table[];
  metadata?: DocumentMetadata;
}

export interface Table {
  id: string;
  rows: TableRow[];
  headers: string[];
  caption?: string;
}

export interface TableRow {
  cells: string[];
}

export interface DocumentMetadata {
  title?: string;
  author?: string;
  page_count?: number;
  created_at?: Date;
  extracted_at: Date;
}

// Tauri Command Types
export interface TauriResponse<T = any> {
  success: boolean;
  data?: T;
  error?: string;
}

// App State Types
export interface AppConfig {
  theme: 'light' | 'dark';
  autoSave: boolean;
  outputFormat: 'json' | 'csv' | 'markdown';
}

export interface LogEntry {
  level: 'info' | 'warn' | 'error' | 'debug';
  message: string;
  timestamp: Date;
}
