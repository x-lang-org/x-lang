//! LSP Server core implementation

use std::sync::Arc;

use anyhow::Result;
use crossbeam_channel::Sender;
use dashmap::DashMap;
use lsp_server::{Connection, ErrorCode, Message, Notification, Request, Response};
use lsp_types::{
    notification::{
        self, DidChangeTextDocument, DidCloseTextDocument, DidOpenTextDocument, Exit,
    },
    request::{Initialize, Shutdown},
    InitializeParams, InitializeResult, ServerCapabilities, TextDocumentSyncCapability,
    TextDocumentSyncKind,
};
use log::{error, info, trace};

use crate::state::WorkspaceState;

/// LSP Server instance
pub struct LspServer {
    connection: Connection,
    workspace: Arc<WorkspaceState>,
    request_handlers: DashMap<String, Box<dyn Fn(Request) -> Result<Response> + Send + Sync>>,
    notification_handlers: DashMap<String, Box<dyn Fn(Notification) -> Result<()> + Send + Sync>>,
}

impl LspServer {
    /// Create a new LSP server instance
    pub fn new() -> Result<Self> {
        let (connection, _io_threads) = Connection::stdio();
        let workspace = Arc::new(WorkspaceState::new());

        let server = Self {
            connection,
            workspace,
            request_handlers: DashMap::new(),
            notification_handlers: DashMap::new(),
        };

        Ok(server)
    }

    /// Register all request and notification handlers
    fn register_handlers(&mut self) {
        // Core lifecycle handlers
        self.register_request_handler::<Initialize>(|req| Ok(Response::new_ok(req.id, initialize_result())));
        self.register_request_handler::<Shutdown>(|req| Ok(Response::new_ok(req.id, ())));

        // Text document handlers
        let workspace = self.workspace();
        let sender = self.sender().clone();
        self.register_notification_handler::<DidOpenTextDocument>(move |notif| {
            let params: lsp_types::DidOpenTextDocumentParams = serde_json::from_value(notif.params)?;
            let uri = params.text_document.uri;
            let content = params.text_document.text;
            let version = params.text_document.version;

            info!("Opened document: {}", uri);
            workspace.open_document(uri.clone(), content, version);

            // Trigger diagnostic update
            crate::handlers::diagnostic::update_diagnostics(&workspace, &sender, &uri)?;

            Ok(())
        });

        let workspace = self.workspace();
        let sender = self.sender().clone();
        self.register_notification_handler::<DidChangeTextDocument>(move |notif| {
            let params: lsp_types::DidChangeTextDocumentParams = serde_json::from_value(notif.params)?;
            let uri = params.text_document.uri;
            let version = params.text_document.version;

            // For full sync, we take the first change's text
            if let Some(change) = params.content_changes.first() {
                info!("Updated document: {}", uri);
                workspace.update_document(uri.clone(), change.text.clone(), version);

                // Trigger diagnostic update
                crate::handlers::diagnostic::update_diagnostics(&workspace, &sender, &uri)?;
            }

            Ok(())
        });

        let workspace = self.workspace();
        self.register_notification_handler::<DidCloseTextDocument>(move |notif| {
            let params: lsp_types::DidCloseTextDocumentParams = serde_json::from_value(notif.params)?;
            let uri = params.text_document.uri;

            info!("Closed document: {}", uri);
            workspace.close_document(&uri);

            Ok(())
        });

        // Register feature handlers
        crate::handlers::register_handlers(self);
    }

    /// Register a request handler
    pub fn register_request_handler<R>(
        &mut self,
        handler: impl Fn(Request) -> Result<Response> + Send + Sync + 'static,
    ) where
        R: lsp_types::request::Request,
    {
        self.request_handlers.insert(R::METHOD.to_string(), Box::new(handler));
    }

    /// Register a notification handler
    pub fn register_notification_handler<N>(
        &mut self,
        handler: impl Fn(Notification) -> Result<()> + Send + Sync + 'static,
    ) where
        N: lsp_types::notification::Notification,
    {
        self.notification_handlers.insert(N::METHOD.to_string(), Box::new(handler));
    }

    /// Run the LSP server main loop
    pub fn run(&mut self) -> Result<()> {
        self.register_handlers();

        // Run initialization
        let initialize_params = self.connection.initialize(serde_json::to_value(initialize_result())?)?;
        let initialize_params: InitializeParams = serde_json::from_value(initialize_params)?;
        info!("Initialized with client: {:?}", initialize_params.client_info);

        // Main message loop
        info!("Entering main message loop");
        while let Ok(msg) = self.connection.receiver.recv() {
            match msg {
                Message::Request(req) => {
                    if self.connection.handle_shutdown(&req)? {
                        return Ok(());
                    }

                    trace!("Received request: {} (id: {:?})", req.method, req.id);

                    if let Some(handler) = self.request_handlers.get(&req.method) {
                        match handler(req) {
                            Ok(resp) => {
                                self.connection.sender.send(Message::Response(resp))?;
                            }
                            Err(e) => {
                                error!("Error handling request: {}", e);
                            }
                        }
                    } else {
                        error!("No handler for request: {}", req.method);
                        let resp = Response::new_err(
                            req.id,
                            ErrorCode::MethodNotFound as i32,
                            format!("Method not found: {}", req.method),
                        );
                        self.connection.sender.send(Message::Response(resp))?;
                    }
                }
                Message::Notification(notif) => {
                    trace!("Received notification: {}", notif.method);

                    if notif.method == <Exit as notification::Notification>::METHOD {
                        return Ok(());
                    }

                    if let Some(handler) = self.notification_handlers.get(&notif.method) {
                        if let Err(e) = handler(notif) {
                            error!("Error handling notification: {}", e);
                        }
                    } else {
                        error!("No handler for notification: {}", notif.method);
                    }
                }
                Message::Response(resp) => {
                    trace!("Received response: {:?}", resp.id);
                }
            }
        }

        Ok(())
    }

    /// Get the workspace state
    pub fn workspace(&self) -> Arc<WorkspaceState> {
        self.workspace.clone()
    }

    /// Get the connection sender for sending messages to the client
    pub fn sender(&self) -> &Sender<Message> {
        &self.connection.sender
    }
}

/// Create the initialize result with server capabilities
fn initialize_result() -> InitializeResult {
    InitializeResult {
        capabilities: ServerCapabilities {
            text_document_sync: Some(TextDocumentSyncCapability::Kind(TextDocumentSyncKind::FULL)),
            completion_provider: Some(lsp_types::CompletionOptions::default()),
            definition_provider: Some(lsp_types::OneOf::Left(true)),
            hover_provider: Some(lsp_types::HoverProviderCapability::Options(lsp_types::HoverOptions::default())),
            references_provider: Some(lsp_types::OneOf::Right(lsp_types::ReferencesOptions {
                work_done_progress_options: Default::default(),
            })),
            document_symbol_provider: Some(lsp_types::OneOf::Right(lsp_types::DocumentSymbolOptions {
                work_done_progress_options: Default::default(),
                label: Some("X Language Symbols".to_string()),
            })),
            ..Default::default()
        },
        server_info: Some(lsp_types::ServerInfo {
            name: "X Language LSP Server".to_string(),
            version: Some(env!("CARGO_PKG_VERSION").to_string()),
        }),
    }
}
