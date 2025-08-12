use std::path::PathBuf;

/// All possible state mutations in the application
#[derive(Debug, Clone)]
pub enum Action {
    // Mouse actions
    MouseDown(u16, u16, MouseButton, crossterm::event::KeyModifiers),
    MouseDrag(u16, u16),
    MouseUp(u16, u16),
    // PDF actions
    OpenPdf(PathBuf),
    PdfLoaded(PdfMetadata),
    NavigatePage(PageDirection),
    SetZoom(ZoomLevel),
    ZoomIn,
    ZoomOut,
    ZoomReset,
    ToggleAutoFit,
    ToggleDarkMode,
    
    // Editor actions
    ExtractMatrix,
    MatrixExtracted(Vec<Vec<char>>),
    ExportMatrix,
    EnterEditMode,
    ExitEditMode,
    InsertChar(char),
    DeleteChar,
    MoveCursor(CursorDirection),
    StartSelection(Position),
    StartBlockSelection(Position),
    UpdateSelection(Position),
    EndSelection,
    Copy,
    Cut,
    Paste(String),
    PasteFromSystem,
    SelectAll,
    DeleteSelection,
    
    // UI actions
    SwitchPanel(Panel),
    ToggleTheme,
    ResizeSplit(i8),
    ShowHelp,
    HideHelp,
    ShowCommandPalette,
    HideCommandPalette,
    
    // Terminal Panel actions
    ToggleTerminalPanel,
    ResizeTerminalPanel(i16),  // Positive = bigger, negative = smaller
    ClearTerminalOutput,
    AddTerminalOutput(String),
    ScrollTerminalUp,
    ScrollTerminalDown,
    SelectTerminalText(usize, usize),  // Start and end line indices
    CopyTerminalSelection,
    
    // Search
    StartSearch,
    UpdateSearchQuery(String),
    SearchNext,
    SearchPrevious,
    EndSearch,
    ClearSearch,
    
    // System
    Error(String),
    SetStatus(String),
    Quit,
}

#[derive(Debug, Clone)]
pub struct PdfMetadata {
    pub path: PathBuf,
    pub page_count: usize,
    pub title: Option<String>,
}

#[derive(Debug, Clone, Copy)]
pub enum PageDirection {
    Next,
    Previous,
    First,
    Last,
    Jump(usize),
}

#[derive(Debug, Clone, Copy)]
pub enum ZoomLevel {
    FitWidth,
    FitHeight,
    Percentage(f32),
    Increase,
    Decrease,
}

#[derive(Debug, Clone, Copy)]
pub enum CursorDirection {
    Up,
    Down,
    Left,
    Right,
    Home,
    End,
    PageUp,
    PageDown,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Position {
    pub row: usize,
    pub col: usize,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Panel {
    Pdf,
    Text,
}

#[derive(Debug, Clone, Copy)]
pub enum MouseButton {
    Left,
    Right,
    Middle,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum SelectionMode {
    Line,        // Traditional line-based selection
    Block,       // Rectangular selection
}