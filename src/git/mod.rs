pub mod diff;

use anyhow::{Context, Result};
use git2::{BranchType, DiffOptions, Repository, StatusOptions, StatusShow};
use std::path::Path;

use crate::app::{BranchInfo, ChangedFile, CommitInfo, FileStatus};

pub fn open_repo(path: &Path) -> Result<Repository> {
    Repository::discover(path).context("Not a git repository or no parent is")
}

pub fn list_changed_files(repo: &Repository) -> Result<Vec<ChangedFile>> {
    let mut opts = StatusOptions::new();
    opts.show(StatusShow::IndexAndWorkdir)
        .include_untracked(true)
        .renames_head_to_index(true);

    let statuses = repo.statuses(Some(&mut opts))?;
    let mut files = Vec::new();

    for entry in statuses.iter() {
        let path = Path::new(entry.path().unwrap_or("<unknown>")).to_path_buf();
        let flags = entry.status();

        let staged = flags.intersects(
            git2::Status::INDEX_NEW
                | git2::Status::INDEX_MODIFIED
                | git2::Status::INDEX_DELETED
                | git2::Status::INDEX_RENAMED
                | git2::Status::INDEX_TYPECHANGE,
        );

        let status =
            if flags.contains(git2::Status::WT_NEW) || flags.contains(git2::Status::INDEX_NEW) {
                FileStatus::Added
            } else if flags.contains(git2::Status::WT_DELETED)
                || flags.contains(git2::Status::INDEX_DELETED)
            {
                FileStatus::Deleted
            } else if flags.contains(git2::Status::WT_RENAMED)
                || flags.contains(git2::Status::INDEX_RENAMED)
            {
                FileStatus::Renamed
            } else if flags.contains(git2::Status::WT_TYPECHANGE)
                || flags.contains(git2::Status::INDEX_TYPECHANGE)
            {
                FileStatus::Copied
            } else {
                FileStatus::Modified
            };

        files.push(ChangedFile {
            path,
            status,
            staged,
        });
    }

    files.sort_by(|a, b| a.path.cmp(&b.path));
    Ok(files)
}

pub fn get_current_branch(repo: &Repository) -> String {
    repo.head()
        .ok()
        .and_then(|h| h.shorthand().map(|s| s.to_string()))
        .unwrap_or_else(|| "detached".into())
}

pub fn list_branches(repo: &Repository) -> Result<Vec<BranchInfo>> {
    let current = get_current_branch(repo);
    let mut branches = Vec::new();

    for branch_type in [BranchType::Local, BranchType::Remote] {
        let iter = repo.branches(Some(branch_type))?;
        for branch in iter.flatten() {
            let (branch, _) = branch;
            if let Some(name) = branch.name().ok().flatten() {
                branches.push(BranchInfo {
                    name: name.to_string(),
                    current: name == current,
                });
            }
        }
    }

    branches.sort_by(|a, b| a.name.cmp(&b.name));
    Ok(branches)
}

pub fn get_commit_history(repo: &Repository, max_count: usize) -> Result<Vec<CommitInfo>> {
    let mut revwalk = repo.revwalk()?;
    revwalk.push_head()?;
    revwalk.set_sorting(git2::Sort::TIME)?;

    let mut commits = Vec::new();
    for oid in revwalk.take(max_count) {
        let oid = oid?;
        let commit = repo.find_commit(oid)?;
        let author = commit.author();
        commits.push(CommitInfo {
            id: oid,
            author: author.name().unwrap_or("unknown").to_string(),
            message: commit
                .message()
                .unwrap_or("")
                .lines()
                .next()
                .unwrap_or("")
                .to_string(),
            time: commit.time().seconds(),
        });
    }
    Ok(commits)
}

pub fn get_commit_files(repo: &Repository, commit_id: git2::Oid) -> Result<Vec<ChangedFile>> {
    let commit = repo.find_commit(commit_id)?;
    let tree = commit.tree()?;
    let parent_tree = commit.parents().next().and_then(|p| p.tree().ok());

    let mut diff_opts = DiffOptions::new();
    let diff = repo.diff_tree_to_tree(parent_tree.as_ref(), Some(&tree), Some(&mut diff_opts))?;

    let mut files = Vec::new();

    for delta in diff.deltas() {
        let path = delta
            .new_file()
            .path()
            .or_else(|| delta.old_file().path())
            .unwrap_or(Path::new("<unknown>"))
            .to_path_buf();

        let status = match delta.status() {
            git2::Delta::Added => FileStatus::Added,
            git2::Delta::Deleted => FileStatus::Deleted,
            git2::Delta::Modified => FileStatus::Modified,
            git2::Delta::Renamed => FileStatus::Renamed,
            git2::Delta::Copied => FileStatus::Copied,
            _ => FileStatus::Modified,
        };

        files.push(ChangedFile {
            path,
            status,
            staged: true,
        });
    }

    Ok(files)
}

pub fn stage_file(repo: &Repository, path: &Path) -> Result<()> {
    let mut index = repo.index()?;
    index.add_path(path)?;
    index.write()?;
    Ok(())
}

pub fn unstage_file(repo: &Repository, path: &Path) -> Result<()> {
    let mut index = repo.index()?;
    index.remove_path(path)?;
    index.write()?;
    Ok(())
}

pub fn stage_all(repo: &Repository) -> Result<()> {
    let mut index = repo.index()?;
    index.add_all(["*"].iter(), git2::IndexAddOption::DEFAULT, None)?;
    index.write()?;
    Ok(())
}

pub fn unstage_all(repo: &Repository) -> Result<()> {
    let head = repo.head()?.peel_to_tree()?;
    let mut index = repo.index()?;
    index.read_tree(&head)?;
    index.write()?;
    Ok(())
}

pub fn create_commit(repo: &Repository, message: &str) -> Result<()> {
    let signature = repo.signature()?;
    let mut index = repo.index()?;
    let tree_id = index.write_tree()?;
    let tree = repo.find_tree(tree_id)?;

    let parent_commit = repo.head().ok().and_then(|h| h.peel_to_commit().ok());

    let parents: Vec<&git2::Commit> = parent_commit.iter().collect();
    repo.commit(
        Some("HEAD"),
        &signature,
        &signature,
        message,
        &tree,
        &parents,
    )?;
    Ok(())
}

pub fn switch_branch(repo: &Repository, name: &str) -> Result<()> {
    let branch = repo.find_branch(name, BranchType::Local)?;
    let branch_ref = branch.get();
    let commit = branch_ref.peel_to_commit()?;
    repo.set_head(branch_ref.name().unwrap())?;
    repo.reset(commit.as_object(), git2::ResetType::Hard, None)?;
    Ok(())
}
