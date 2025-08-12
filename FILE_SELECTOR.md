# Custom File Selector for Chonker6

## ✨ What We Built

A **beautiful, native file selector** that matches the app's aesthetic perfectly:

![Features]
- 🎨 **Matches app theme** - Background highlighting, no borders
- 📁 **Smart filtering** - Shows only PDFs and directories
- 📊 **File sizes** - Shows PDF sizes (KB/MB)
- ⌨️ **Keyboard navigation** - Arrow keys, Enter, Esc, Backspace
- 🌈 **Visual feedback** - Selected item highlighted in bright colors
- 📂 **Directory browsing** - Navigate your entire file system

## 🎯 Design Choices

### Background Highlighting Instead of Borders
```rust
// Selected item - bright highlight
(Color::Rgb(255, 255, 200), Color::Rgb(60, 65, 78))

// PDF files - slightly highlighted  
(Color::Rgb(150, 200, 255), dialog_bg)

// Directories - normal
(Color::Rgb(180, 180, 200), dialog_bg)
```

### Overlay Effect
- Dark semi-transparent background
- Centered dialog (70% width, 80% height)
- Clean, modern appearance

## 🎮 Controls

| Key | Action |
|-----|--------|
| `Ctrl+O` | Open file selector |
| `↑/↓` | Navigate items |
| `Enter` | Open file/directory |
| `Backspace` | Go to parent directory |
| `Esc` | Cancel selection |

## 📝 Code Structure

```rust
pub struct FileSelector {
    pub current_path: PathBuf,
    pub entries: Vec<FileEntry>,
    pub selected_index: usize,
    pub active: bool,
    filter_extension: Option<String>, // "pdf"
}
```

## 🚀 How It Works

1. **Activation**: Press `Ctrl+O` to activate
2. **Directory Reading**: Scans current directory for PDFs and folders
3. **Sorting**: Directories first, then files, alphabetically
4. **Rendering**: Overlay with custom colors matching app theme
5. **Selection**: Returns `PathBuf` when PDF selected

## 🎨 Visual Design

```
╔══════════════════════════════════════════╗
║ 📂 /Users/jack/Documents                 ║  <- Current path
╠══════════════════════════════════════════╣
║ 📁 ..                                     ║  <- Parent directory
║ 📁 Projects                               ║  <- Directory
║ 📄 report.pdf (2MB)                       ║  <- PDF with size
║ [📄 presentation.pdf (5MB)]               ║  <- Selected (highlighted)
║ 📁 Archives                               ║  <- Another directory
╠══════════════════════════════════════════╣
║ ↑↓ Navigate • Enter: Open • Esc: Cancel  ║  <- Help text
╚══════════════════════════════════════════╝
```

## 💡 Why This is Better Than Native Dialogs

1. **Consistent UI** - Matches your app perfectly
2. **Faster** - No external process spawning
3. **Filtered** - Shows only relevant files (PDFs)
4. **Keyboard-driven** - No mouse required
5. **Customizable** - Easy to add features like preview, search, etc.

## 🔮 Future Enhancements

- [ ] Search/filter as you type
- [ ] Recent files list
- [ ] Bookmarked directories
- [ ] PDF preview on hover
- [ ] Multi-select support
- [ ] Breadcrumb navigation