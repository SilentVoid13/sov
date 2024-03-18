use std::path::PathBuf;
use std::sync::{Arc, Mutex};

use dashmap::DashMap;
use ropey::Rope;
use tower_lsp::jsonrpc::Result;
use tower_lsp::lsp_types::*;
use tower_lsp::{Client, LanguageServer};
use tower_lsp::{LspService, Server};
use sov::note::{Link, SovNote};
use sov::Sov;

pub struct SovLanguageServer {
    pub client: Client,
    pub sov: Arc<Mutex<Sov>>,
    pub document_map: DashMap<String, Rope>,
}

#[tower_lsp::async_trait]
impl LanguageServer for SovLanguageServer {
    async fn initialize(&self, _: InitializeParams) -> Result<InitializeResult> {
        let params = InitializeResult {
            server_info: None,
            capabilities: ServerCapabilities {
                text_document_sync: Some(TextDocumentSyncCapability::Kind(
                    TextDocumentSyncKind::FULL,
                )),
                completion_provider: Some(CompletionOptions {
                    resolve_provider: Some(false),
                    trigger_characters: Some(vec!["[".into(), "#".into()]),
                    work_done_progress_options: Default::default(),
                    all_commit_characters: None,
                    completion_item: None,
                }),
                definition_provider: Some(OneOf::Left(true)),
                references_provider: Some(OneOf::Left(true)),
                ..Default::default()
            },
        };
        Ok(params)
    }

    async fn initialized(&self, _: InitializedParams) {
        self.client
            .log_message(MessageType::ERROR, "server initialized!")
            .await;
    }

    async fn shutdown(&self) -> Result<()> {
        Ok(())
    }

    async fn did_open(&self, params: DidOpenTextDocumentParams) {
        self.client
            .log_message(MessageType::ERROR, "file opened!")
            .await;
        self.on_change(params.text_document).await;
    }

    async fn did_change(&self, mut params: DidChangeTextDocumentParams) {
        self.client
            .log_message(MessageType::ERROR, "file opened!")
            .await;
        self.on_change(TextDocumentItem {
            uri: params.text_document.uri,
            text: std::mem::take(&mut params.content_changes[0].text),
            // TODO: what should i put here?
            language_id: "".into(),
            version: params.text_document.version,
        })
        .await;
    }

    async fn did_save(&self, _: DidSaveTextDocumentParams) {
        // TODO: no unwrap?
        self.sov.lock().unwrap().index().unwrap();

        self.client
            .log_message(MessageType::ERROR, "file saved!")
            .await;
    }

    async fn did_close(&self, _: DidCloseTextDocumentParams) {
        self.client
            .log_message(MessageType::ERROR, "file closed!")
            .await;
    }

    async fn goto_definition(
        &self,
        params: GotoDefinitionParams,
    ) -> Result<Option<GotoDefinitionResponse>> {
        let res = async {
            let uri = params.text_document_position_params.text_document.uri;
            let rope = self.document_map.get(uri.as_str())?;
            let position = params.text_document_position_params.position;
            let line = rope.get_line(position.line as usize)?;

            let links = SovNote::parse_links(line.as_str()?).ok()?;
            // get closest link to cursor
            let link = links.iter().min_by(|l1, l2| {
                let cmp1 = ((l1.start.ch as i64 - position.character as i64).abs())
                    .min((l1.end.ch as i64 - position.character as i64).abs());
                let cmp2 = ((l2.start.ch as i64 - position.character as i64).abs())
                    .min((l2.end.ch as i64 - position.character as i64).abs());
                cmp1.cmp(&cmp2)
            })?;

            let sov = self.sov.lock().unwrap();
            let note_path = sov.resolve_note(&link.value).ok()??;
            let note_uri = Self::path_to_uri(&note_path).ok()?;

            let range = Range::default();
            Some(GotoDefinitionResponse::Scalar(Location::new(
                note_uri, range,
            )))
        }
        .await;
        Ok(res)
    }

    async fn completion(&self, params: CompletionParams) -> Result<Option<CompletionResponse>> {
        self.client
            .log_message(MessageType::ERROR, "completion triggered!")
            .await;
        let uri = params.text_document_position.text_document.uri;
        let position = params.text_document_position.position;

        let completions = || -> Option<Vec<CompletionItem>> {
            let rope = self.document_map.get(&uri.to_string())?;
            let line = rope.get_line(position.line as usize)?;
            match &line.as_str()?.as_bytes()[..position.character as usize] {
                &[.., b'[', b'['] => {
                    let notes = self.sov.lock().unwrap().list_note_names().ok()?;
                    let mut ret = Vec::new();
                    for note in notes {
                        let completion = CompletionItem {
                            label: note,
                            kind: Some(CompletionItemKind::FILE),
                            ..Default::default()
                        };
                        ret.push(completion);
                    }
                    Some(ret)
                }
                &[.., b'#'] => {
                    let tags = self.sov.lock().unwrap().list_tags().ok()?;
                    let mut ret = Vec::new();
                    for tag in tags {
                        let tag = format!("#{}", tag);
                        let completion = CompletionItem {
                            label: tag,
                            kind: Some(CompletionItemKind::CONSTANT),
                            ..Default::default()
                        };
                        ret.push(completion);
                    }
                    Some(ret)
                }
                _ => return None,
            }
        }();

        Ok(completions.map(|c| CompletionResponse::List(CompletionList {
            is_incomplete: false,
            items: c,
        })))
    }

    /*
    async fn completion_resolve(&self, params: CompletionItem) -> Result<CompletionItem> {
        todo!()
    }
    */

    async fn references(&self, params: ReferenceParams) -> Result<Option<Vec<Location>>> {
        self.client
            .log_message(MessageType::ERROR, "references triggered!")
            .await;

        let uri = params.text_document_position.text_document.uri;
        let position = params.text_document_position.position;

        let references = || -> Option<Vec<Location>> {
            let rope = self.document_map.get(&uri.to_string())?;
            let line = rope.get_line(position.line as usize)?;
            let filename = if let Some(link) = Self::link_under_cursor(&position, line.as_str()?) {
                link.value
            } else {
                let path = Self::uri_to_path(&uri).ok()?;
                path.file_stem()?.to_str()?.to_string()
            };
            let mut ret = Vec::new();
            let backlinks = self.sov.lock().unwrap().list_backlinks(&filename).ok()?;
            for backlink in backlinks {
                let uri = Self::path_to_uri(&backlink).ok()?;
                let location = Location::new(uri, Range::default());
                ret.push(location);
            }
            Some(ret)
        }();
        Ok(references)
    }
}

impl SovLanguageServer {
    async fn on_change(&self, params: TextDocumentItem) {
        self.client
            .log_message(MessageType::ERROR, "on_change triggered!")
            .await;
        let rope = ropey::Rope::from_str(&params.text);
        self.document_map
            .insert(params.uri.to_string(), rope.clone());
    }

    fn path_to_uri(path: &PathBuf) -> Result<Url> {
        // TODO
        let uri = Url::from_file_path(path).unwrap();
        Ok(uri)
    }

    fn uri_to_path(uri: &Url) -> Result<PathBuf> {
        // TODO
        let path = uri.to_file_path().unwrap();
        Ok(path)
    }

    fn link_under_cursor(position: &Position, line: &str) -> Option<Link> {
        let links = SovNote::parse_links(line).ok()?;
        for link in links {
            if link.start.ch <= position.character as u64
                && link.end.ch >= position.character as u64
            {
                return Some(link);
            }
        }
        None
    }
}

#[tokio::main]
pub async fn main() -> Result<()> {
    let stdin = tokio::io::stdin();
    let stdout = tokio::io::stdout();
    let sov = Sov::new().unwrap();
    let (service, socket) = LspService::new(|client| SovLanguageServer {
        client,
        sov: Arc::new(Mutex::new(sov)),
        document_map: Default::default(),
    });
    Server::new(stdin, stdout, socket).serve(service).await;
    Ok(())
}