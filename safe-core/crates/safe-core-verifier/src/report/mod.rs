use serde::{Deserialize, Serialize};
use crate::checks::Issue;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileReport {
    pub path: String,
    pub language: String,
    pub alpha_hat: f64,
    pub passed: bool,
    pub issues: Vec<Issue>,
    pub suggestions: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GlobalReport {
    pub total_files: usize,
    pub passed_files: usize,
    pub failed_files: usize,
    pub file_reports: Vec<FileReport>,
}

impl GlobalReport {
    pub fn from_file_reports(file_reports: Vec<FileReport>) -> Self {
        let total = file_reports.len();
        let passed = file_reports.iter().filter(|r| r.passed).count();
        Self {
            total_files: total,
            passed_files: passed,
            failed_files: total - passed,
            file_reports,
        }
    }
}
