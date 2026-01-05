use bstr::{BStr, ByteSlice};
use futures::StreamExt;
use jj_lib::backend::TreeValue;
use jj_lib::commit::Commit;
use jj_lib::diff_presentation::unified::{unified_diff_hunks, DiffLineType};
use jj_lib::diff_presentation::LineCompareMode;
use jj_lib::matchers::EverythingMatcher;
use jj_lib::merge::{Diff, MergedTreeValue};
use jj_lib::repo::Repo;
use std::fmt::Write;
use std::sync::Arc;
use tokio::io::AsyncReadExt;

use crate::error::JjaiError;

const CONTEXT_LINES: usize = 3;

pub async fn render_commit_patch<R: Repo>(repo: &R, commit: &Commit) -> Result<String, JjaiError> {
    let parents: Vec<_> = commit
        .parents()
        .collect::<Result<Vec<_>, _>>()
        .map_err(|e| JjaiError::Diff(e.to_string()))?;

    let parent_tree = if parents.is_empty() {
        repo.store().empty_merged_tree()
    } else {
        parents[0].tree()
    };

    let commit_tree = commit.tree();

    let mut output = String::new();

    let diff_stream = parent_tree.diff_stream(&commit_tree, &EverythingMatcher);
    let entries: Vec<_> = diff_stream.collect().await;

    for entry in entries {
        let path = entry.path;
        let diff_values = entry
            .values
            .map_err(|e| JjaiError::Diff(e.to_string()))?;

        writeln!(
            output,
            "diff --git a/{} b/{}",
            path.as_internal_file_string(),
            path.as_internal_file_string()
        )
        .map_err(|e| JjaiError::Diff(e.to_string()))?;

        let before_content = get_content(repo.store(), &diff_values.before).await?;
        let after_content = get_content(repo.store(), &diff_values.after).await?;

        let before_bstr: &BStr = before_content.as_bytes().as_bstr();
        let after_bstr: &BStr = after_content.as_bytes().as_bstr();
        let contents = Diff::new(before_bstr, after_bstr);

        let hunks = unified_diff_hunks(contents, CONTEXT_LINES, LineCompareMode::Exact);

        for hunk in hunks {
            let left_start = hunk.left_line_range.start + 1;
            let left_len = hunk.left_line_range.len();
            let right_start = hunk.right_line_range.start + 1;
            let right_len = hunk.right_line_range.len();

            writeln!(
                output,
                "@@ -{},{} +{},{} @@",
                left_start, left_len, right_start, right_len
            )
            .map_err(|e| JjaiError::Diff(e.to_string()))?;

            for (line_type, tokens) in &hunk.lines {
                let prefix = match line_type {
                    DiffLineType::Context => " ",
                    DiffLineType::Removed => "-",
                    DiffLineType::Added => "+",
                };

                let line_content: String = tokens
                    .iter()
                    .map(|(_, bytes)| bytes.to_str_lossy())
                    .collect();

                write!(output, "{}{}", prefix, line_content)
                    .map_err(|e| JjaiError::Diff(e.to_string()))?;
            }
        }
    }

    Ok(output)
}

async fn get_content(
    store: &Arc<jj_lib::store::Store>,
    value: &MergedTreeValue,
) -> Result<String, JjaiError> {
    if value.is_absent() {
        return Ok(String::new());
    }

    let resolved = value.as_resolved();
    if let Some(Some(TreeValue::File { id, .. })) = resolved {
        let mut reader = store
            .read_file(&jj_lib::repo_path::RepoPath::root(), id)
            .await
            .map_err(|e| JjaiError::Diff(e.to_string()))?;

        let mut buf = Vec::new();
        AsyncReadExt::read_to_end(&mut reader, &mut buf)
            .await
            .map_err(|e| JjaiError::Diff(e.to_string()))?;

        Ok(String::from_utf8_lossy(&buf).to_string())
    } else {
        Ok(String::new())
    }
}
