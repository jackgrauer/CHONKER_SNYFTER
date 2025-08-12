# Chonker6 - Clean Architecture TUI

## 🎯 What We've Built (Phase 1 Complete!)

A **properly architected** PDF viewer TUI with:
- ✅ **Modular structure** - No more 2,249-line files!
- ✅ **Redux-like state** - Predictable, testable state transitions
- ✅ **Clean event handling** - Mode-based, no spaghetti
- ✅ **Visual highlighting** - Background colors instead of borders
- ✅ **Component isolation** - Each panel is independent

## 📊 Architecture Comparison

| Aspect | Chonker5 (Old) | Chonker6 (New) |
|--------|----------------|----------------|
| Main file | 2,249 lines | 88 lines |
| Architecture | Monolithic | Modular |
| State management | Mutable chaos | Immutable + Actions |
| Event handling | 500-line match | Clean pipeline |
| Testability | Nearly impossible | Easy to test |
| Bug potential | High | Low |

## 🏗️ Current Structure

```
chonker6/
├── src/
│   ├── main.rs         (88 lines)  - Just setup!
│   ├── app.rs          (250 lines) - Orchestration
│   ├── actions.rs      (90 lines)  - All possible actions
│   └── state/
│       ├── app_state.rs   (100 lines)
│       ├── pdf_state.rs   (50 lines)
│       ├── editor_state.rs (75 lines)
│       └── ui_state.rs    (20 lines)
```

**Total: ~673 lines** vs **2,249 lines** in Chonker5!

## 🎨 Visual Design

Instead of borders, we use **background highlighting**:
- **Focused panel**: Bright background (RGB 40,44,52)
- **Unfocused panel**: Dark background (RGB 20,22,26)
- **Status bar**: Green/Red tinted based on state

## 🚀 Running

```bash
./run.sh
```

Or directly:
```bash
./target/release/chonker6
```

## ⌨️ Current Controls

- `Tab` - Switch between panels
- `Ctrl+H` - Show help
- `Ctrl+Q` - Quit
- `←/→` or `h/l` - Navigate pages (when PDF loaded)
- Arrow keys - Navigate in editor (when editing)

## 📈 Next Steps (Phases 2-5)

### Phase 2: PDF Viewing
- [ ] Integrate PDFium
- [ ] Image rendering (iTerm2/Kitty)
- [ ] Zoom controls

### Phase 3: Text Extraction  
- [ ] Spatial extraction algorithm
- [ ] Matrix display
- [ ] Cursor system

### Phase 4: Editing
- [ ] Full text editing
- [ ] Copy/paste
- [ ] Search

### Phase 5: Polish
- [ ] Animations
- [ ] Themes
- [ ] Config file

## 💡 Why This Architecture Wins

1. **No more coordinate overflows** - Geometry module handles bounds
2. **No more event conflicts** - Clean pipeline with modes
3. **No more state bugs** - Immutable state transitions
4. **Easy to add features** - Just add new actions and handlers
5. **Easy to test** - Pure functions everywhere

## 🎯 The Vibe

Clean. Modular. Maintainable. 

This is what Chonker5 should have been from the start!