use bstr::ByteSlice;
use futures::StreamExt;
use jj_lib::backend::TreeValue;
use jj_lib::commit::Commit;
use jj_lib::diff::{diff, DiffHunkKind};
use jj_lib::matchers::EverythingMatcher;
use jj_lib::merge::MergedTreeValue;
use jj_lib::repo::Repo;
use std::fmt::Write;
use std::sync::Arc;
use tokio::io::AsyncReadExt;

use crate::error::JjaiError;

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

        let hunks = diff([before_content.as_bytes(), after_content.as_bytes()]);

        for hunk in hunks {
            match hunk.kind {
                DiffHunkKind::Matching => {
                    if let Some(content) = hunk.contents.first() {
                        for line in content.lines() {
                            writeln!(output, " {}", line.to_str_lossy())
                                .map_err(|e| JjaiError::Diff(e.to_string()))?;
                        }
                    }
                }
                DiffHunkKind::Different => {
                    if let Some(old) = hunk.contents.first() {
                        for line in old.lines() {
                            writeln!(output, "-{}", line.to_str_lossy())
                                .map_err(|e| JjaiError::Diff(e.to_string()))?;
                        }
                    }
                    if let Some(new) = hunk.contents.get(1) {
                        for line in new.lines() {
                            writeln!(output, "+{}", line.to_str_lossy())
                                .map_err(|e| JjaiError::Diff(e.to_string()))?;
                        }
                    }
                }
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
