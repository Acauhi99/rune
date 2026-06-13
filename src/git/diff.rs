use std::path::Path;
use anyhow::Result;
use git2::{DiffOptions, Repository};

use crate::app::{DiffLine, DiffLineKind, FileDiff, Hunk};

pub fn get_workdir_diff(repo: &Repository, path: &Path) -> Result<FileDiff> {
    let mut opts = DiffOptions::new();
    opts.pathspec(path);

    let diff = repo.diff_index_to_workdir(None, Some(&mut opts))?;

    let old_path = path.to_path_buf();
    let new_path = path.to_path_buf();

    let hunks = diff_to_hunks(&diff, path)?;

    Ok(FileDiff {
        old_path,
        new_path,
        hunks,
    })
}

pub fn get_staged_diff(repo: &Repository, path: &Path) -> Result<FileDiff> {
    let head_tree = repo
        .head()
        .ok()
        .and_then(|h| h.peel_to_tree().ok());

    let mut opts = DiffOptions::new();
    opts.pathspec(path);

    let diff = repo.diff_tree_to_index(
        head_tree.as_ref(),
        None,
        Some(&mut opts),
    )?;

    let old_path = path.to_path_buf();
    let new_path = path.to_path_buf();

    let hunks = diff_to_hunks(&diff, path)?;

    Ok(FileDiff {
        old_path,
        new_path,
        hunks,
    })
}

pub fn get_commit_diff(repo: &Repository, commit_id: git2::Oid) -> Result<Vec<FileDiff>> {
    let commit = repo.find_commit(commit_id)?;
    let tree = commit.tree()?;
    let parent_tree = commit.parents().next().and_then(|p| p.tree().ok());

    let diff = repo.diff_tree_to_tree(
        parent_tree.as_ref(),
        Some(&tree),
        None,
    )?;

    let mut files = Vec::new();

    for (delta_idx, delta) in diff.deltas().enumerate() {
        let path = delta
            .new_file()
            .path()
            .or_else(|| delta.old_file().path())
            .unwrap_or(Path::new("<unknown>"))
            .to_path_buf();

        let old_path = delta.old_file().path().unwrap_or(&path).to_path_buf();
        let new_path = delta.new_file().path().unwrap_or(&path).to_path_buf();
        let hunks = diff_patch_to_hunks(&diff, delta_idx)?;

        files.push(FileDiff {
            old_path,
            new_path,
            hunks,
        });
    }

    Ok(files)
}

fn diff_to_hunks(diff: &git2::Diff, file_path: &Path) -> Result<Vec<Hunk>> {
    let mut hunks = Vec::new();

    for (delta_idx, delta) in diff.deltas().enumerate() {
        let delta_path = delta
            .new_file()
            .path()
            .or_else(|| delta.old_file().path());

        if delta_path != Some(file_path) {
            continue;
        }

        extract_hunks(diff, delta_idx, &mut hunks)?;
    }

    Ok(hunks)
}

fn diff_patch_to_hunks(diff: &git2::Diff, delta_index: usize) -> Result<Vec<Hunk>> {
    let mut hunks = Vec::new();
    extract_hunks(diff, delta_index, &mut hunks)?;
    Ok(hunks)
}

fn extract_hunks(diff: &git2::Diff, delta_index: usize, hunks: &mut Vec<Hunk>) -> Result<()> {
    let patch = git2::Patch::from_diff(diff, delta_index)?;
    if let Some(patch) = patch {
        for hunk_idx in 0..patch.num_hunks() {
            let (hunk, num_lines) = patch.hunk(hunk_idx)?;
            let mut lines = Vec::new();

            for line_idx in 0..num_lines {
                let line = patch.line_in_hunk(hunk_idx, line_idx)?;
                let kind = match line.origin() {
                    '+' => DiffLineKind::Add,
                    '-' => DiffLineKind::Delete,
                    _ => DiffLineKind::Context,
                };

                let content = String::from_utf8_lossy(line.content()).to_string();
                let old_lineno = line.old_lineno();
                let new_lineno = line.new_lineno();

                lines.push(DiffLine {
                    kind,
                    content,
                    old_lineno,
                    new_lineno,
                });
            }

            hunks.push(Hunk {
                header: String::from_utf8_lossy(hunk.header()).to_string(),
                lines,
            });
        }
    }
    Ok(())
}

pub fn get_side_by_side(hunk: &Hunk) -> Vec<(Option<&DiffLine>, Option<&DiffLine>)> {
    let mut pairs: Vec<(Option<&DiffLine>, Option<&DiffLine>)> = Vec::new();
    let mut old_lines: Vec<&DiffLine> = Vec::new();
    let mut new_lines: Vec<&DiffLine> = Vec::new();

    for line in &hunk.lines {
        match line.kind {
            DiffLineKind::Delete => old_lines.push(line),
            DiffLineKind::Add => new_lines.push(line),
            DiffLineKind::Context => {
                if !old_lines.is_empty() || !new_lines.is_empty() {
                    let max = old_lines.len().max(new_lines.len());
                    for i in 0..max {
                        pairs.push((
                            old_lines.get(i).copied(),
                            new_lines.get(i).copied(),
                        ));
                    }
                    old_lines.clear();
                    new_lines.clear();
                }
                pairs.push((Some(line), Some(line)));
            }
        }
    }

    if !old_lines.is_empty() || !new_lines.is_empty() {
        let max = old_lines.len().max(new_lines.len());
        for i in 0..max {
            pairs.push((
                old_lines.get(i).copied(),
                new_lines.get(i).copied(),
            ));
        }
    }

    pairs
}
