# X Language VSCode Extension

X Language support for Visual Studio Code.

## Features

- **Syntax Highlighting** - Support for all X language keywords, types, and operators
- **LSP Features**:
  - Diagnostic errors (parsing errors)
  - Hover information for functions and variables
  - Go to definition
  - Find references
  - Code completion (keywords, types, built-in functions, symbols from current file)
  - Document symbols / Outline view

## Requirements

- [X Language LSP Server](https://github.com/xiongdi/x-lang) - The LSP server must be built and accessible
- Node.js (for extension development)
- VSCode 1.80.0 or later

## Building the LSP Server

Before using the extension, you need to build the X Language LSP server:

```bash
cd tools/x-lsp
cargo build
```

## Installation

### From Source

1. Clone the X Language repository
2. Build the LSP server: `cargo build -p x-lsp`
3. Open `tools/x-lang-vscode` in VSCode
4. Press F5 to run the extension in development mode

### Configuration

You can configure the LSP server path in VSCode settings:

```json
{
  "xLanguage.serverPath": "path/to/x-lsp"
}
```

If not configured, the extension will look for the LSP server at:
- `<workspace>/target/debug/x-lsp` (Unix)
- `<workspace>/target/debug/x-lsp.exe` (Windows)

## Usage

1. Open a `.x` file in VSCode
2. The extension will automatically start the LSP server
3. Syntax highlighting will be applied automatically
4. Use LSP features (hover, go to definition, etc.)

## Extension Points

### Syntax Highlighting

The extension uses TextMate grammar for syntax highlighting, defined in `syntaxes/x.tmLanguage.json`.

### Language Configuration

Language-specific settings like comments, brackets, and auto-closing pairs are defined in `language-configuration.json`.

## Commands

- `xLanguage.restart` - Restart the LSP server

## Development

```bash
# Install dependencies
npm install

# Compile TypeScript
npm run compile

# Watch for changes
npm run watch
```

## License

MIT OR Apache-2.0 OR BSD-3-Clause
