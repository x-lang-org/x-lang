# X Language Syntax Highlight Generator

Tool to automatically generate syntax highlighting definitions for various editors and IDEs, directly from the X language compiler's lexer definitions. This ensures perfect consistency between the compiler and editor syntax highlighting.

## Supported Editors

- ✅ VS Code (TextMate grammar)
- ✅ Vim
- [ ] Neovim (Tree-sitter grammar)
- [ ] Sublime Text
- [ ] Emacs
- [ ] JetBrains IDEs (IntelliJ, CLion, etc.)

## Installation

```bash
cd tools/x-syntax-gen
cargo build --release
```

The binary will be available at `target/release/x-syntax-gen`.

## Usage

### Generate all syntax definitions

```bash
x-syntax-gen all --output ./output
```

### Generate for a specific editor

```bash
# VS Code
x-syntax-gen vscode --output ./output

# Vim
x-syntax-gen vim --output ./output
```

### VS Code Installation

1. Copy the generated `x.tmLanguage.json` file to your VS Code extension directory
2. Or create a simple extension with the generated grammar

### Vim Installation

1. Copy the generated `x.vim` file to `~/.vim/syntax/`
2. Add the following to your `.vimrc`:
   ```vim
   autocmd BufRead,BufNewFile *.x set filetype=x
   ```

## Development

The generator works by:
1. Importing token definitions directly from `x-lexer`
2. Mapping compiler tokens to generic syntax types
3. Generating editor-specific syntax files using Handlebars templates

### Adding support for a new editor

1. Create a new generator module in `src/generators/`
2. Add a template file in `templates/`
3. Register the command in `src/main.rs`

## License

Dual-licensed under MIT, Apache-2.0, or BSD-3-Clause.
