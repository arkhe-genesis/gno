pub mod languages;
pub mod registry;
pub mod checks;
pub mod report;

use anyhow::Result;
use std::path::{Path, PathBuf};
use std::collections::HashMap;
use tree_sitter::Parser;

use registry::{LanguageRegistry, registry};
use safe_core_utils::CgfEngine;
use checks::{FileContext, Check, CheckResult, Issue, IssueCategory, Severity, AllChecks};
use report::{FileReport, GlobalReport};

pub struct PolyglotVerifier {
    parsers: HashMap<String, Parser>,
    cgf: CgfEngine,
    all_checks: AllChecks,
}

impl PolyglotVerifier {
    pub fn new() -> Result<Self> {
        let mut parsers = HashMap::new();

        let reg = registry();
        for plugin in reg.all() {
            if let Some(ts_lang) = plugin.tree_sitter_language() {
                let mut parser = Parser::new();
                parser.set_language(&ts_lang)?;
                parsers.insert(plugin.id().to_string(), parser);
            }
        }

        let all_checks = AllChecks(vec![
            Box::new(checks::UniversalConventionXCheck),
            Box::new(checks::UniversalSafetyCheck),
            Box::new(checks::UniversalDependencyCheck),
        ]);

        Ok(Self {
            parsers,
            cgf: CgfEngine::new(100),
            all_checks,
        })
    }

    pub async fn verify_file(&mut self, path: &Path) -> Result<FileReport> {
        let code = std::fs::read_to_string(path)?;
        let reg = registry();
        let plugin = reg.detect(path)?;

        let ts_lang = plugin.tree_sitter_language()
            .ok_or_else(|| anyhow::anyhow!("No tree-sitter support for {:?}", path))?;

        let parser = self.parsers.get_mut(plugin.id())
            .ok_or_else(|| anyhow::anyhow!("Parser not found for {:?}", path))?;

        let tree = parser.parse(&code, None)
            .ok_or_else(|| anyhow::anyhow!("Parse failed for {:?}", path))?;

        let ctx = FileContext {
            path: path.to_path_buf(),
            code,
            tree,
            content_hash: 0,
            plugin,
        };

        let result = self.all_checks.execute(&ctx).await?;

        Ok(FileReport {
            path: path.display().to_string(),
            language: plugin.id().to_string(),
            alpha_hat: 1.0,
            passed: result.passed,
            issues: result.issues,
            suggestions: result.suggestions,
        })
    }
}
