# X Language LSP Server

Language Server Protocol (LSP) implementation for the X programming language.

## Features

- [ ] Real-time syntax and semantic error diagnostics
- [ ] Code completion (keywords, variables, functions)
- [ ] Hover information (type signatures, documentation)
- [ ] Go to definition
- [ ] Find references
- [ ] Document symbol outline
- [ ] Rename symbol
- [ ] Formatting

## Installation

```bash
cd tools/x-lsp
cargo build --release
```

The binary will be available at `target/release/x-lsp`.

## Usage

### VS Code

1. Install the X Language extension
2. The extension will automatically start the LSP server when opening `.x` files

### Neovim

Add the following to your Neovim configuration:

```lua
local lspconfig = require 'lspconfig'
local configs = require 'lspconfig.configs'

if not configs.x_lsp then
  configs.x_lsp = {
    default_config = {
      cmd = { '/path/to/x-lsp' },
      filetypes = { 'x' },
      root_dir = function(fname)
        return lspconfig.util.find_git_ancestor(fname)
      end,
      settings = {},
    },
  }
end

lspconfig.x_lsp.setup {}
```

### Other Editors

Configure your editor to use the `x-lsp` binary as the LSP server for files with `.x` extension.

## Development

The LSP server is built using:
- `lsp-server` and `lsp-types` crates for LSP protocol handling
- Reuses the existing X language compiler components (`x-lexer`, `x-parser`, `x-typechecker`)

### Running Tests

```bash
cargo test
```

## License

Dual-licensed under MIT, Apache-2.0, or BSD-3-Clause.
