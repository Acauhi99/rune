use git2::Oid;
use std::path::PathBuf;

pub struct ChangedFile {
    pub path: PathBuf,
    pub status: FileStatus,
    pub staged: bool,
}

#[allow(dead_code)]
pub enum FileStatus {
    Added,
    Modified,
    Deleted,
    Renamed,
    Copied,
    Untracked,
}

#[allow(dead_code)]
impl FileStatus {
    pub fn label(&self) -> &'static str {
        match self {
            FileStatus::Added => "A",
            FileStatus::Modified => "M",
            FileStatus::Deleted => "D",
            FileStatus::Renamed => "R",
            FileStatus::Copied => "C",
            FileStatus::Untracked => "?",
        }
    }
}

#[allow(dead_code)]
pub struct CommitInfo {
    pub id: Oid,
    pub author: String,
    pub message: String,
    pub time: i64,
}

pub struct BranchInfo {
    pub name: String,
    #[allow(dead_code)]
    pub current: bool,
}

pub struct DiffLine {
    pub kind: DiffLineKind,
    pub content: String,
    pub old_lineno: Option<u32>,
    pub new_lineno: Option<u32>,
}

#[derive(Clone, PartialEq)]
pub enum DiffLineKind {
    Add,
    Delete,
    Context,
}

pub struct Hunk {
    pub header: String,
    pub lines: Vec<DiffLine>,
}

#[allow(dead_code)]
pub struct FileDiff {
    pub old_path: PathBuf,
    pub new_path: PathBuf,
    pub hunks: Vec<Hunk>,
}

impl FileDiff {
    pub fn is_empty(&self) -> bool {
        self.hunks.is_empty()
    }
}

#[allow(dead_code)]
pub enum AppMode {
    Tree,
    Diff,
    CommitLog,
    Help,
}

pub enum PanelFocus {
    Tree,
    Diff,
}

pub struct RuneApp {
    #[allow(dead_code)]
    pub repo_path: PathBuf,
    pub mode: AppMode,
    pub focus: PanelFocus,
    pub changed_files: Vec<ChangedFile>,
    pub selected_file: usize,
    pub diff: Option<FileDiff>,
    pub diff_scroll: u16,
    pub commits: Vec<CommitInfo>,
    pub selected_commit: usize,
    pub branches: Vec<BranchInfo>,
    pub current_branch: String,
    pub show_help: bool,
    pub filter_text: String,
    pub filter_active: bool,
}

impl RuneApp {
    pub fn new(repo_path: PathBuf) -> Self {
        Self {
            repo_path,
            mode: AppMode::Tree,
            focus: PanelFocus::Tree,
            changed_files: Vec::new(),
            selected_file: 0,
            diff: None,
            diff_scroll: 0,
            commits: Vec::new(),
            selected_commit: 0,
            branches: Vec::new(),
            current_branch: String::new(),
            show_help: false,
            filter_text: String::new(),
            filter_active: false,
        }
    }

    pub fn filtered_files(&self) -> Vec<&ChangedFile> {
        if !self.filter_active || self.filter_text.is_empty() {
            return self.changed_files.iter().collect();
        }
        let lower = self.filter_text.to_lowercase();
        self.changed_files
            .iter()
            .filter(|f| f.path.to_string_lossy().to_lowercase().contains(&lower))
            .collect()
    }

    pub fn selected_file_entry(&self) -> Option<&ChangedFile> {
        let filtered = self.filtered_files();
        filtered.get(self.selected_file).copied()
    }
}
