### Known Issues with Git Commits in Warp Terminal

#### Background
The Warp terminal has a known issue where using multi-line git commit messages with the `-m` flag can cause cascading output and terminal corruption. This document aims to capture and inform users about specific patterns that are problematic and provide safe alternatives.

#### Unsafe Patterns
- **Multi-line strings:**
  ```bash
  git commit -m "First line
  Second line"
  ```
  The above will cause terminal issues.

- **Unclosed Quotes:**
  ```bash
  git commit -m "Opening quote without closing
  ```

#### Safe Alternatives
- **Use Editor:**
  Open the editor for writing the commit message, which handles multi-line input safely.
  ```bash
  git commit -e
  ```

- **Single Line Messages:**
  Ensures message is within a single line and properly quoted.
  ```bash
  git commit -m "A straightforward single line message"
  ```

#### Prevention
Ensure to validate your commit message before using the `-m` flag. Use our automated workflows to catch potential issues automatically or opt for editor mode by default.

For more details and automation scripts, refer to our safe-commit.yaml in `.warp/workflows/`.
