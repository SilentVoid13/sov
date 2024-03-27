use std::path::PathBuf;
use std::sync::{Arc, Mutex};

use dashmap::DashMap;
use linkify::{LinkFinder, LinkKind};
use ropey::Rope;
use sov_core::note::{Link, SovNote};
use sov_core::Sov;
use tower_lsp::jsonrpc::Result;
use tower_lsp::lsp_types::*;
use tower_lsp::{Client, LanguageServer, LspService, Server};

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
                execute_command_provider: Some(ExecuteCommandOptions {
                    commands: vec![
                        "sov.index".into(),
                        "sov.daily".into(),
                    ],
                    ..Default::default()
                }),
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
        let uri = &params.text_document.uri;
        let text = &params.text_document.text;
        self.on_change(text, uri).await;
        let rope = self
            .document_map
            .get(params.text_document.uri.as_str())
            .unwrap();
        self.refresh_diagnostics(&params.text_document.uri, &rope)
            .await;
    }

    async fn did_change(&self, mut params: DidChangeTextDocumentParams) {
        self.client
            .log_message(MessageType::ERROR, "file opened!")
            .await;
        let text = std::mem::take(&mut params.content_changes[0].text);
        let uri = &params.text_document.uri;
        self.on_change(&text, uri).await;
    }

    async fn did_save(&self, params: DidSaveTextDocumentParams) {
        self.client
            .log_message(MessageType::ERROR, "file saved!")
            .await;
        // refresh metadata
        self.sov.lock().unwrap().index().unwrap();
        let uri = params.text_document.uri;
        let rope = self.document_map.get(uri.as_str()).unwrap();
        self.refresh_diagnostics(&uri, &rope).await;
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
                let cmp1 = ((l1.start as i64 - position.character as i64).abs())
                    .min((l1.end as i64 - position.character as i64).abs());
                let cmp2 = ((l2.start as i64 - position.character as i64).abs())
                    .min((l2.end as i64 - position.character as i64).abs());
                cmp1.cmp(&cmp2)
            });
            if let Some(link) = link {
                let sov = self.sov.lock().unwrap();
                let note_path = sov.resolve_note(&link.value).ok()??;
                let note_uri = Self::path_to_uri(&note_path).ok()?;

                let range = Range::default();
                Some(GotoDefinitionResponse::Scalar(Location::new(
                    note_uri, range,
                )))
            } else {
                // Try to parse URLs
                let finder = LinkFinder::new();
                let link = finder.links(line.as_str()?).next()?;

                if link.kind() == &LinkKind::Url {
                    std::process::Command::new("xdg-open")
                        .arg(link.as_str())
                        .spawn()
                        .ok()?;
                }
                None
            }
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
            match line.as_str()?.as_bytes()[..position.character as usize] {
                [.., b'[', b'['] => {
                    let mut ret = Vec::new();

                    let note_filenames = self.sov.lock().unwrap().list_note_names().ok()?;
                    for filename in note_filenames {
                        let ins_text = format!("{}]]", filename);
                        let completion = CompletionItem {
                            label: filename,
                            insert_text: Some(ins_text),
                            kind: Some(CompletionItemKind::FILE),
                            ..Default::default()
                        };
                        ret.push(completion);
                    }

                    let note_aliases = self.sov.lock().unwrap().list_note_aliases().ok()?;
                    for (filename, alias) in note_aliases {
                        let ins_text = format!("{}|{}]]", filename, alias);
                        let completion = CompletionItem {
                            label: alias,
                            insert_text: Some(ins_text),
                            kind: Some(CompletionItemKind::FILE),
                            ..Default::default()
                        };
                        ret.push(completion);
                    }

                    Some(ret)
                }
                [.., b'#'] => {
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
                _ => None,
            }
        }();

        Ok(completions.map(|c| {
            CompletionResponse::List(CompletionList {
                is_incomplete: false,
                items: c,
            })
        }))
    }

    async fn references(&self, params: ReferenceParams) -> Result<Option<Vec<Location>>> {
        self.client
            .log_message(MessageType::ERROR, "references triggered!")
            .await;

        let uri = params.text_document_position.text_document.uri;
        let position = params.text_document_position.position;

        let references = async {
            let rope = self.document_map.get(&uri.to_string())?;
            let line = rope.get_line(position.line as usize)?;
            let filename = if let Some(link) = Self::link_under_cursor(&position, line.as_str()?) {
                link.value
            } else {
                let path = Self::uri_to_path(&uri).ok()?;
                path.file_stem()?.to_str()?.to_string()
            };
            let mut ret = Vec::new();
            let backlinks = self.sov.lock().unwrap().resolve_backlinks(&filename).ok()?;
            for (path, link) in backlinks {
                let uri = Self::path_to_uri(&path).ok()?;
                // TODO: improve this?
                let new_rope = ropey::Rope::from_reader(std::fs::File::open(&path).ok()?).ok()?;
                let start_pos = Self::offset_to_position(link.start, &new_rope);
                let end_pos = Self::offset_to_position(link.end, &new_rope);
                let range = Range {
                    start: start_pos,
                    end: end_pos,
                };
                let location = Location::new(uri, range);
                ret.push(location);
            }
            Some(ret)
        }.await;
        Ok(references)
    }

    async fn execute_command(&self, params: ExecuteCommandParams) -> Result<Option<serde_json::Value>> {
        self.client
            .log_message(MessageType::ERROR, "execute_command triggered!")
            .await;
        let command = params.command.as_str();
        match command {
            "sov.index" => {
                // TODO: remove unwrap
                self.sov.lock().unwrap().index().unwrap();
                Ok(None)
            }
            "sov.daily" => {
                // TODO: remove unwrap
                let daily_path = self.sov.lock().unwrap().daily().unwrap();
                let daily_path = daily_path.to_str().unwrap().to_string();
                Ok(Some(daily_path.into()))
            }
            "sov.list.tags" => {
                let tags = self.sov.lock().unwrap().list_tags().unwrap();
                Ok(Some(tags.into()))
            }
            _ => Ok(None),
        }
    }
}

impl SovLanguageServer {
    async fn on_change(&self, text: &str, uri: &Url) {
        self.client
            .log_message(MessageType::ERROR, "on_change triggered!")
            .await;
        let rope = ropey::Rope::from_str(text);
        let uri = uri.to_string();
        self.document_map.insert(uri, rope);
    }

    async fn refresh_diagnostics(&self, uri: &Url, rope: &Rope) {
        let diagnostics = async {
            let path = Self::uri_to_path(uri).ok()?;
            let filename = path.file_stem()?.to_str()?;

            let dead_links = self.sov.lock().unwrap().resolve_dead_links(filename).ok()?;
            let mut diagnostics = Vec::new();
            for dead_link in dead_links {
                let start_pos = Self::offset_to_position(dead_link.start, rope);
                let end_pos = Self::offset_to_position(dead_link.end, rope);
                let diagnostic = Diagnostic {
                    range: Range {
                        start: start_pos,
                        end: end_pos,
                    },
                    severity: Some(DiagnosticSeverity::INFORMATION),
                    message: "Unresolved Reference".into(),
                    ..Default::default()
                };
                diagnostics.push(diagnostic);
            }
            Some(diagnostics)
        }
        .await;
        if let Some(diagnostics) = diagnostics {
            self.client
                .publish_diagnostics(uri.clone(), diagnostics, None)
                .await;
        }
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
        links.into_iter().find(|link| {
            position.character as usize >= link.start && position.character as usize <= link.end
        })
    }

    fn offset_to_position(offset: usize, rope: &Rope) -> Position {
        let line = rope.char_to_line(offset);
        let character = offset - rope.line_to_char(line);
        Position::new(line as u32, character as u32)
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
