use lsp_types::Command;

use crate::{prelude::*, SemanticRequest};

/// The [`textDocument/codeLens`] request is sent from the client to the server
/// to compute code lenses for a given text document.
///
/// [`textDocument/codeLens`]: https://microsoft.github.io/language-server-protocol/specification#textDocument_codeLens
#[derive(Debug, Clone)]
pub struct CodeLensRequest {
    /// The path of the document to request for.
    pub path: PathBuf,
}

impl SemanticRequest for CodeLensRequest {
    type Response = Vec<CodeLens>;

    fn request(self, ctx: &mut LocalContext) -> Option<Self::Response> {
        let source = ctx.source_by_path(&self.path).ok()?;

        let mut res = vec![];

        let doc_start = ctx.to_lsp_range(0..0, &source);
        let doc_lens = |title: &str, args: Vec<JsonValue>| CodeLens {
            range: doc_start,
            command: Some(Command {
                title: title.to_string(),
                command: "tinymist.runCodeLens".to_string(),
                arguments: Some(args),
            }),
            data: None,
        };

        res.push(doc_lens(
            &tinymist_l10n::t!("tinymist-query.code-action.profile", "Profile"),
            vec!["profile".into()],
        ));
        res.push(doc_lens(
            &tinymist_l10n::t!("tinymist-query.code-action.preview", "Preview"),
            vec!["preview".into()],
        ));

        let is_html = ctx.world.library.features.is_enabled(typst::Feature::Html);

        if is_html {
            res.push(doc_lens(
                &tinymist_l10n::t!("tinymist-query.code-action.exportHtml", "Export HTML"),
                vec!["export-html".into()],
            ));
        } else {
            res.push(doc_lens(
                &tinymist_l10n::t!("tinymist-query.code-action.exportPdf", "Export PDF"),
                vec!["export-pdf".into()],
            ));
        }

        res.push(doc_lens(
            &tinymist_l10n::t!("tinymist-query.code-action.more", "More .."),
            vec!["more".into()],
        ));

        Some(res)
    }
}
