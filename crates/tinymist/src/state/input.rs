use std::path::PathBuf;

use lsp_types::request::WorkspaceConfiguration;
use lsp_types::*;
use once_cell::sync::OnceCell;
use reflexo_typst::Bytes;
use serde_json::{Map, Value as JsonValue};
use sync_lsp::*;
use tinymist_project::{Interrupt, ProjectResolutionKind};
use tinymist_query::{to_typst_range, PositionEncoding};
use tinymist_std::error::{prelude::*, IgnoreLogging};
use tinymist_std::ImmutPath;
use typst::{diag::FileResult, syntax::Source};

use crate::route::ProjectResolution;
use crate::task::FormatterConfig;
use crate::world::vfs::{notify::MemoryEvent, FileChangeSet};
use crate::world::TaskInputs;
use crate::{init::*, *};

/// LSP Document Synchronization
impl ServerState {
    pub(crate) fn did_open(&mut self, params: DidOpenTextDocumentParams) -> LspResult<()> {
        log::info!("did open {:?}", params.text_document.uri);
        let path = as_path_(params.text_document.uri);
        let text = params.text_document.text;

        self.create_source(path.clone(), text)
            .map_err(|e| invalid_params(e.to_string()))?;

        // Focus after opening
        self.implicit_focus_entry(|| Some(path.as_path().into()), 'o');
        Ok(())
    }

    pub(crate) fn did_close(&mut self, params: DidCloseTextDocumentParams) -> LspResult<()> {
        let path = as_path_(params.text_document.uri);

        self.remove_source(path.clone())
            .map_err(|e| invalid_params(e.to_string()))?;
        Ok(())
    }

    pub(crate) fn did_change(&mut self, params: DidChangeTextDocumentParams) -> LspResult<()> {
        let path = as_path_(params.text_document.uri);
        let changes = params.content_changes;

        self.edit_source(path.clone(), changes, self.const_config().position_encoding)
            .map_err(|e| invalid_params(e.to_string()))?;
        Ok(())
    }

    pub(crate) fn did_save(&mut self, _params: DidSaveTextDocumentParams) -> LspResult<()> {
        Ok(())
    }
}

/// LSP Configuration Synchronization
impl ServerState {
    pub(crate) fn on_changed_configuration(
        &mut self,
        values: Map<String, JsonValue>,
    ) -> LspResult<()> {
        let old_config = self.config.clone();
        match self.config.update_by_map(&values) {
            Ok(()) => {}
            Err(err) => {
                self.config = old_config;
                log::error!("error applying new settings: {err}");
                return Err(invalid_params(format!(
                    "error applying new settings: {err}"
                )));
            }
        }

        let new_export_config = self.config.export();
        if old_config.export() != new_export_config {
            self.change_export_config(new_export_config);
        }

        if old_config.compile.primary_opts() != self.config.compile.primary_opts() {
            self.config.compile.fonts = OnceCell::new(); // todo: don't reload fonts if not changed
            self.restart_primary()
                .log_error("could not restart primary");
        }

        if old_config.semantic_tokens != self.config.semantic_tokens {
            self.enable_sema_token_caps(self.config.semantic_tokens == SemanticTokensMode::Enable)
                .log_error("could not change semantic tokens config");
        }

        let new_formatter_config = self.config.formatter();
        if !old_config.formatter().eq(&new_formatter_config) {
            let enabled = !matches!(new_formatter_config.config, FormatterConfig::Disable);
            self.enable_formatter_caps(enabled)
                .log_error("could not change formatter config");

            self.formatter.change_config(new_formatter_config);
        }

        log::info!("new settings applied");
        Ok(())
    }

    pub(crate) fn did_change_configuration(
        &mut self,
        params: DidChangeConfigurationParams,
    ) -> LspResult<()> {
        // For some clients, we don't get the actual changed configuration and need to
        // poll for it https://github.com/microsoft/language-server-protocol/issues/676
        if let JsonValue::Object(settings) = params.settings {
            return self.on_changed_configuration(settings);
        };

        self.client.send_request::<WorkspaceConfiguration>(
            ConfigurationParams {
                items: Config::get_items(),
            },
            Self::workspace_configuration_callback,
        );
        Ok(())
    }

    fn workspace_configuration_callback(this: &mut ServerState, resp: lsp_server::Response) {
        if let Some(err) = resp.error {
            log::error!("failed to request configuration: {err:?}");
            return;
        }

        let Some(result) = resp.result else {
            log::error!("no configuration returned");
            return;
        };

        let Some(resp) = serde_json::from_value::<Vec<JsonValue>>(result)
            .log_error("could not parse configuration")
        else {
            return;
        };
        let _ = this.on_changed_configuration(Config::values_to_map(resp));
    }
}

/// In memory source file management.
impl ServerState {
    /// Updates a set of source files.
    fn update_sources(&mut self, files: FileChangeSet) -> Result<()> {
        let intr = Interrupt::Memory(MemoryEvent::Update(files.clone()));
        self.project.interrupt(intr);

        Ok(())
    }

    /// Creates a new source file.
    pub fn create_source(&mut self, path: PathBuf, content: String) -> Result<()> {
        let path: ImmutPath = path.into();

        log::info!("create source: {path:?}");
        self.memory_changes
            .insert(path.clone(), Source::detached(content.clone()));

        let content: Bytes = content.as_bytes().into();

        // todo: is it safe to believe that the path is normalized?
        let files = FileChangeSet::new_inserts(vec![(path, FileResult::Ok(content).into())]);

        self.update_sources(files)
    }

    /// Removes a source file.
    pub fn remove_source(&mut self, path: PathBuf) -> Result<()> {
        let path: ImmutPath = path.into();

        self.memory_changes.remove(&path);
        log::info!("remove source: {path:?}");

        // todo: is it safe to believe that the path is normalized?
        let files = FileChangeSet::new_removes(vec![path]);

        self.update_sources(files)
    }

    /// Edits a source file.
    pub fn edit_source(
        &mut self,
        path: PathBuf,
        content: Vec<TextDocumentContentChangeEvent>,
        position_encoding: PositionEncoding,
    ) -> Result<()> {
        let path: ImmutPath = path.into();

        let source = self
            .memory_changes
            .get_mut(&path)
            .ok_or_else(|| error_once!("file missing", path: path.display()))?;

        for change in content {
            let replacement = change.text;
            match change.range {
                Some(lsp_range) => {
                    let range = to_typst_range(lsp_range, position_encoding, source)
                        .expect("invalid range");
                    source.edit(range, &replacement);
                }
                None => {
                    source.replace(&replacement);
                }
            }
        }

        let snapshot = FileResult::Ok(source.text().as_bytes().into()).into();

        let files = FileChangeSet::new_inserts(vec![(path.clone(), snapshot)]);

        self.update_sources(files)
    }

    /// Queries a source file that must be in memory.
    pub fn query_source<T>(
        &self,
        path: ImmutPath,
        f: impl FnOnce(Source) -> Result<T>,
    ) -> Result<T> {
        let snapshot = self.memory_changes.get(&path);
        let snapshot = snapshot.ok_or_else(|| anyhow::anyhow!("file missing {path:?}"))?;
        let source = snapshot.clone();
        f(source)
    }
}

/// Main file mutations on the primary project (which is used for the language
/// queries.)
impl ServerState {
    /// Changes main file to the given path.
    pub fn change_main_file(&mut self, path: Option<ImmutPath>) -> Result<bool> {
        if path
            .as_deref()
            .is_some_and(|p| !p.is_absolute() && !p.starts_with("/untitled"))
        {
            return Err(error_once!("entry file must be absolute", path: path.unwrap().display()));
        }

        let task = self.resolve_task_or(path);

        log::info!("the task of the primary is changing to {task:?}");

        let id = self.project.primary_id().clone();
        self.project.interrupt(Interrupt::ChangeTask(id, task));

        Ok(true)
    }

    /// Pins the main file to the given path
    pub fn pin_main_file(&mut self, new_entry: Option<ImmutPath>) -> Result<()> {
        self.pinning = new_entry.is_some();
        let entry = new_entry
            .or_else(|| self.entry_resolver().resolve_default())
            .or_else(|| self.focusing.clone());

        self.change_main_file(entry).map(|_| ())
    }

    /// Focuses main file to the given path.
    pub fn focus_main_file(&mut self, new_entry: Option<ImmutPath>) -> Result<bool> {
        if self.pinning || self.config.compile.has_default_entry_path {
            self.focusing = new_entry;
            return Ok(false);
        }

        self.change_main_file(new_entry.clone())
    }

    /// This is used for tracking activating document status if a client is not
    /// performing any focus command request.
    ///
    /// See <https://github.com/microsoft/language-server-protocol/issues/718>
    ///
    /// we do want to focus the file implicitly by `textDocument/diagnostic`
    /// (pullDiagnostics mode), as suggested by language-server-protocol#718,
    /// however, this has poor support, e.g. since neovim 0.10.0.
    pub fn implicit_focus_entry(
        &mut self,
        new_entry: impl FnOnce() -> Option<ImmutPath>,
        site: char,
    ) {
        if self.ever_manual_focusing {
            return;
        }
        // didOpen
        match site {
            // foldingRange, hover, semanticTokens
            'f' | 'h' | 't' => {
                self.ever_focusing_by_activities = true;
            }
            // didOpen
            _ => {
                if self.ever_focusing_by_activities {
                    return;
                }
            }
        }

        let new_entry = new_entry();

        let update_result = self.focus_main_file(new_entry.clone());
        match update_result {
            Ok(true) => {
                log::info!("file focused[implicit,{site}]: {new_entry:?}");
            }
            Err(err) => {
                log::warn!("could not focus file: {err}");
            }
            Ok(false) => {}
        }
    }
}

/// Task input resolution.
impl ServerState {
    fn resolve_task_without_lock(&self, path: Option<ImmutPath>) -> TaskInputs {
        TaskInputs {
            entry: Some(self.entry_resolver().resolve(path)),
            ..Default::default()
        }
    }

    pub(crate) fn resolve_task_or(&mut self, path: Option<ImmutPath>) -> TaskInputs {
        path.clone()
            .map(|path| self.resolve_task(path))
            .unwrap_or_else(|| self.resolve_task_without_lock(path))
    }

    pub(crate) fn resolve_task(&mut self, path: ImmutPath) -> TaskInputs {
        let proj_input = matches!(
            self.config.project_resolution,
            ProjectResolutionKind::LockDatabase
        )
        .then(|| {
            let resolution = self.route.resolve(&path)?;
            let lock = self.route.locate(&resolution)?;

            let ProjectResolution {
                lock_dir,
                project_id,
            } = &resolution;

            let input = lock.get_document(project_id)?;
            let root = input
                .root
                .as_ref()
                .and_then(|res| Some(res.to_abs_path(lock_dir)?.as_path().into()))
                .unwrap_or_else(|| lock_dir.clone());
            let main = input
                .main
                .to_abs_path(lock_dir)
                .map(|path| path.as_path().into())
                .unwrap_or_else(|| path.clone());
            let entry = self
                .entry_resolver()
                .resolve_with_root(Some(root), Some(main));
            log::info!("resolved task with state: {path:?} -> {project_id:?} -> {entry:?}");

            Some(TaskInputs {
                entry: Some(entry),
                ..Default::default()
            })
        });

        proj_input
            .flatten()
            .unwrap_or_else(|| self.resolve_task_without_lock(Some(path)))
    }
}
