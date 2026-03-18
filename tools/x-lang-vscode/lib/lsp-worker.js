// LSP Worker for X Language WASM LSP Server
// This worker loads the WASM module and communicates with the main extension

importScripts('./x-lsp.js');

let wasmModule = null;
let stdinBuffer = '';
let pendingResolve = null;

// Initialize WASM module
async function initWasm() {
    try {
        wasmModule = await xLsp();
        console.log('X Language WASM module initialized');
        return true;
    } catch (err) {
        console.error('Failed to initialize WASM:', err);
        return false;
    }
}

// Handle messages from main thread
self.onmessage = async function(event) {
    const { type, data, id } = event.data;

    switch (type) {
        case 'init':
            const success = await initWasm();
            self.postMessage({ type: 'init-result', success, id });
            break;

        case 'stdin':
            // Forward data to LSP server
            if (wasmModule) {
                try {
                    // This is a simplified version - actual implementation
                    // would need proper JSON-RPC handling
                    wasmModule.process_input(data);
                } catch (err) {
                    console.error('WASM processing error:', err);
                }
            }
            break;

        default:
            console.log('Unknown message type:', type);
    }
};

// Signal ready
console.log('X Language LSP Worker loaded');
