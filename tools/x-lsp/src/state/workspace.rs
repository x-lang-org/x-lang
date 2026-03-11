//! Workspace state management

use std::collections::HashMap;

use dashmap::DashMap;
use lsp_types::Url;

use super::Document;

/// Manages the state of the entire workspace
#[derive(Debug, Default)]
pub struct WorkspaceState {
    /// Open documents, keyed by URI
    documents: DashMap<Url, Document>,
}

impl WorkspaceState {
    /// Create a new workspace state
    pub fn new() -> Self {
        Self {
            documents: DashMap::new(),
        }
    }

    /// Open a new document
    pub fn open_document(&self, uri: Url, content: String, version: i32) {
        let doc = Document::new(uri.clone(), content, version);
        self.documents.insert(uri, doc);
    }

    /// Update an existing document
    pub fn update_document(&self, uri: Url, content: String, version: i32) -> bool {
        if let Some(mut doc) = self.documents.get_mut(&uri) {
            doc.update(content, version);
            true
        } else {
            false
        }
    }

    /// Close a document
    pub fn close_document(&self, uri: &Url) -> bool {
        self.documents.remove(uri).is_some()
    }

    /// Get a document by URI
    pub fn get_document(&self, uri: &Url) -> Option<dashmap::mapref::one::Ref<'_, Url, Document>> {
        self.documents.get(uri)
    }

    /// Get all open documents
    pub fn all_documents(&self) -> Vec<Document> {
        self.documents.iter().map(|entry| entry.value().clone()).collect()
    }

    /// Check if a document is open
    pub fn is_document_open(&self, uri: &Url) -> bool {
        self.documents.contains_key(uri)
    }

    /// Get the number of open documents
    pub fn document_count(&self) -> usize {
        self.documents.len()
    }
}
