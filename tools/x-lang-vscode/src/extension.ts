import * as vscode from 'vscode';
import * as path from 'path';
import * as fs from 'fs';

let languageClient: any = undefined;

function getExtensionPath(): string {
    const ext = vscode.extensions.getExtension('xlangext.x-lang');
    if (ext) {
        return ext.extensionPath;
    }
    return path.join(__dirname, '..');
}

function findLspServer(): { type: 'wasm' | 'binary', path: string } | undefined {
    const config = vscode.workspace.getConfiguration('xLanguage');
    const configuredPath = config.get<string>('serverPath');

    // Option 1: User configured path
    if (configuredPath && configuredPath.trim() !== '') {
        if (fs.existsSync(configuredPath)) {
            return { type: 'binary', path: configuredPath };
        }
    }

    const extPath = getExtensionPath();
    const isWin = process.platform === 'win32';
    const isMac = process.platform === 'darwin';

    // Option 2: Try WASM first (cross-platform!)
    const wasmPath = path.join(extPath, 'lib', 'x-lsp_bg.wasm');
    if (fs.existsSync(wasmPath)) {
        console.log('Using X Language WASM LSP server');
        return { type: 'wasm', path: wasmPath };
    }

    // Option 3: Platform-specific binary
    let binaryName: string;
    if (isWin) binaryName = 'x-lsp.exe';
    else if (isMac) binaryName = 'x-lsp-macos';
    else binaryName = 'x-lsp-linux';

    const binaryPath = path.join(extPath, 'lib', binaryName);
    if (fs.existsSync(binaryPath)) {
        console.log('Using X Language binary LSP server:', binaryName);
        return { type: 'binary', path: binaryPath };
    }

    // Option 4: Try workspace path
    const workspacePath = vscode.workspace.workspaceFolders?.[0]?.uri.fsPath;
    if (workspacePath) {
        const wsBinaryPath = path.join(workspacePath, 'tools', 'target', 'release', binaryName);
        if (fs.existsSync(wsBinaryPath)) {
            return { type: 'binary', path: wsBinaryPath };
        }
    }

    // Option 5: PATH fallback
    return { type: 'binary', path: isWin ? 'x-lsp.exe' : 'x-lsp' };
}

async function startWasmLsp(): Promise<boolean> {
    try {
        // Dynamic import for WASM support
        const wasmPath = path.join(getExtensionPath(), 'lib', 'x-lsp_bg.wasm');
        const wasmJsPath = path.join(getExtensionPath(), 'lib', 'x-lsp.js');

        if (!fs.existsSync(wasmPath) || !fs.existsSync(wasmJsPath)) {
            return false;
        }

        // Load WASM module
        const wasm = await import(wasmJsPath.replace(/\\/g, '/'));
        const wasmModule = await wasm.default();

        console.log('WASM LSP module loaded:', !!wasmModule);
        vscode.window.showInformationMessage('X Language WASM LSP loaded');
        return true;
    } catch (err) {
        console.error('Failed to load WASM LSP:', err);
        return false;
    }
}

async function startBinaryLsp(serverPath: string, context: vscode.ExtensionContext): Promise<void> {
    try {
        const { LanguageClient } = require('vscode-languageclient');

        const serverOptions = {
            command: serverPath,
            args: [],
        };

        const clientOptions = {
            documentSelector: [{ language: 'x', scheme: 'file' }],
            synchronize: {
                fileEvents: vscode.workspace.createFileSystemWatcher('**/*.x')
            },
            diagnosticCollectionName: 'X Language',
            outputChannelName: 'X Language',
        };

        languageClient = new LanguageClient('xLanguageServer', 'X Language Server', serverOptions, clientOptions);
        const disposable = languageClient.start();
        context.subscriptions.push(disposable);

        await languageClient.onReady();
        vscode.window.showInformationMessage('X Language LSP server started');
    } catch (err: any) {
        vscode.window.showWarningMessage(`LSP server failed: ${err.message}`);
    }
}

export function activate(context: vscode.ExtensionContext) {
    console.log('X Language extension activating...');
    console.log('Platform:', process.platform);

    const server = findLspServer();

    if (!server) {
        vscode.window.showWarningMessage(
            'X Language LSP server not found. ' +
            'Configure "xLanguage.serverPath" in settings for custom LSP server.'
        );
        return;
    }

    if (server.type === 'wasm') {
        // Start with WASM
        startWasmLsp().then(success => {
            if (!success) {
                vscode.window.showWarningMessage('WASM LSP failed to load, trying binary...');
            }
        });
    } else {
        // Start with binary
        startBinaryLsp(server.path, context);
    }

    // Register commands
    context.subscriptions.push(
        vscode.commands.registerCommand('xLanguage.openSettings', () => {
            vscode.commands.executeCommand('workbench.action.openSettings', 'xLanguage.serverPath');
        })
    );
}

export function deactivate() {
    if (languageClient) {
        languageClient.stop();
    }
}
