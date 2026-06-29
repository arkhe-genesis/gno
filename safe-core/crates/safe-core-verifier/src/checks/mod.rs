use std::path::PathBuf;
use async_trait::async_trait;
use crate::registry::LanguagePlugin;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum Severity {
    Info,
    Warning,
    Error,
    Critical,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum IssueCategory {
    ConventionX,
    Dependency,
    Safety,
    Other,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Issue {
    pub line: u32,
    pub column: u32,
    pub severity: Severity,
    pub message: String,
    pub category: IssueCategory,
}

pub struct FileContext {
    pub path: PathBuf,
    pub code: String,
    pub tree: tree_sitter::Tree,
    pub content_hash: u64,
    pub plugin: &'static dyn LanguagePlugin,
}

#[derive(Debug, Clone, Default)]
pub struct CheckResult {
    pub passed: bool,
    pub issues: Vec<Issue>,
    pub suggestions: Vec<String>,
    pub score: f64,
}

#[async_trait]
pub trait Check: Send + Sync {
    fn name(&self) -> &str;
    fn category(&self) -> IssueCategory;
    async fn execute(&self, ctx: &FileContext) -> anyhow::Result<CheckResult>;
}

pub struct AllChecks(pub Vec<Box<dyn Check>>);

#[async_trait]
impl Check for AllChecks {
    fn name(&self) -> &str { "all-checks" }
    fn category(&self) -> IssueCategory { IssueCategory::Other }

    async fn execute(&self, ctx: &FileContext) -> anyhow::Result<CheckResult> {
        let mut all_issues = Vec::new();
        let mut all_suggestions = Vec::new();
        let mut total_score = 0.0;
        let mut count = 0;

        for check in &self.0 {
            let result = check.execute(ctx).await?;
            all_issues.extend(result.issues);
            all_suggestions.extend(result.suggestions);
            total_score += result.score;
            count += 1;
        }

        Ok(CheckResult {
            passed: all_issues.is_empty(),
            issues: all_issues,
            suggestions: all_suggestions,
            score: if count > 0 { total_score / count as f64 } else { 1.0 },
        })
    }
}

mod universal;
pub use universal::*;
