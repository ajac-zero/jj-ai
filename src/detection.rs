use jj_lib::backend::CommitId;
use jj_lib::repo::{ReadonlyRepo, Repo};
use std::collections::{HashMap, HashSet};
use std::sync::Arc;

use crate::error::JjaiError;

pub fn find_rewritten_commits(
    before: &Arc<ReadonlyRepo>,
    after: &Arc<ReadonlyRepo>,
) -> Result<Vec<(CommitId, CommitId)>, JjaiError> {
    let mut before_change_to_commits: HashMap<_, HashSet<_>> = HashMap::new();
    for commit_id in before.view().heads().iter() {
        if let Ok(commit) = before.store().get_commit(commit_id) {
            before_change_to_commits
                .entry(commit.change_id().clone())
                .or_default()
                .insert(commit.id().clone());
        }
    }

    let mut after_change_to_commits: HashMap<_, HashSet<_>> = HashMap::new();
    for commit_id in after.view().heads().iter() {
        if let Ok(commit) = after.store().get_commit(commit_id) {
            after_change_to_commits
                .entry(commit.change_id().clone())
                .or_default()
                .insert(commit.id().clone());
        }
    }

    let mut rewrites = Vec::new();

    for (change_id, after_commits) in after_change_to_commits.iter() {
        if after_commits.len() != 1 {
            continue;
        }

        if let Some(before_commits) = before_change_to_commits.get(change_id) {
            if before_commits.len() != 1 {
                continue;
            }

            let old_id = before_commits.iter().next().unwrap();
            let new_id = after_commits.iter().next().unwrap();

            if old_id != new_id {
                rewrites.push((old_id.clone(), new_id.clone()));
            }
        }
    }

    Ok(rewrites)
}
