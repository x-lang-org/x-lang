// Type declarations for vscode-languageclient

declare module 'vscode-languageclient' {
    import * as vscode from 'vscode';

    export interface LanguageClientOptions {
        documentSelector?: vscode.DocumentSelector;
        synchronize?: {
            fileEvents?: vscode.FileSystemWatcher;
        };
        diagnosticCollectionName?: string;
        outputChannelName?: string;
    }

    export interface ServerOptions {
        command: string;
        args?: string[];
        options?: {
            env?: { [key: string]: string };
            stdio?: string | string[];
        };
    }

    export class BaseLanguageClient {
        constructor(
            id: string,
            name: string,
            serverOptions: ServerOptions,
            clientOptions: LanguageClientOptions
        );
        start(): vscode.Disposable;
        onReady(): Promise<void>;
        stop(): Promise<void>;
    }

    export class LanguageClient extends BaseLanguageClient {
        constructor(
            id: string,
            name: string,
            serverOptions: ServerOptions,
            clientOptions: LanguageClientOptions
        );
    }
}
