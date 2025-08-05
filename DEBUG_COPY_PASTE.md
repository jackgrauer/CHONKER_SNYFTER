# Debug Copy/Paste

## What to Test

1. **Run the app**: `cargo run`
2. **Load a PDF** and click PROCESS
3. **Click on the Matrix tab**
4. **Click on a cell** to select it

## Test Copy
1. Press **Cmd+C** (Mac) or **Ctrl+C** (Windows/Linux)
2. Watch the **terminal** for: `🔤 COPY KEY PRESSED`
3. Check the **debug panel** for copy message

## Test Paste
1. Copy some text from anywhere
2. Click on a cell in the matrix
3. Press **Cmd+V** (Mac) or **Ctrl+V** (Windows/Linux)
4. Watch the **terminal** for: `📋 PASTE EVENT DETECTED: X chars`

## Test Direct Typing
1. Click on a cell
2. Type any character
3. Watch the **terminal** for: `🔤 TYPED 'X' at cell (x, y)`
4. The character should update immediately

## What the Debug Panel Shows
- `Focus: MatrixView` - Matrix has keyboard focus
- `Selected: Some((x, y))` - A cell is selected
- `EditMode: false` - Should always be false now (no dialog mode)

If copy/paste keys aren't showing messages in the terminal, then the keyboard events are being intercepted elsewhere.