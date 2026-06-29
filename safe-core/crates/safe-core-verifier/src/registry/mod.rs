use std::collections::HashMap;
use std::path::Path;
use once_cell::sync::Lazy;
use tree_sitter::Language as TsLanguage;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum AnalysisTier {
    CompileTime,
    Runtime,
    ParseOnly,
}

#[derive(Debug, Clone)]
pub struct ConventionXStyle {
    pub function_prefix: &'static str,
    pub param_prefix: &'static str,
    pub struct_suffix: &'static str,
    pub public_only: bool,
    pub description: &'static str,
}

#[derive(Debug, Clone)]
pub struct ExtractedImport {
    pub package: String,
    pub full_path: String,
    pub line: u32,
    pub is_wildcard: bool,
}

#[derive(Debug, Clone)]
pub struct UnsafePattern {
    pub id: &'static str,
    pub query: &'static str,
    pub message: &'static str,
    pub severity: crate::checks::Severity,
    pub auto_fixable: bool,
}

pub trait LanguagePlugin: Send + Sync {
    fn id(&self) -> &str;
    fn display_name(&self) -> &str;
    fn tier(&self) -> AnalysisTier;
    fn extensions(&self) -> &[&str];
    fn tree_sitter_language(&self) -> Option<TsLanguage>;
    fn convention_x_style(&self) -> ConventionXStyle;
    fn import_query(&self) -> &str;
    fn frontier_function_query(&self) -> &str;
    fn unsafe_pattern_queries(&self) -> &[UnsafePattern];
    fn extract_package_from_match(&self, code: &str, capture_text: &str) -> Option<ExtractedImport>;
    fn detect_by_content(&self, _code: &str) -> bool { false }
}

pub struct LanguageRegistry {
    plugins: Vec<Box<dyn LanguagePlugin>>,
    by_id: HashMap<String, usize>,
    by_ext: HashMap<String, usize>,
}

impl LanguageRegistry {
    fn build() -> Self {
        let mut reg = Self {
            plugins: Vec::new(),
            by_id: HashMap::new(),
            by_ext: HashMap::new(),
        };

        reg.register(Box::new(crate::languages::rust::RustPlugin));

        reg
    }

    fn register(&mut self, plugin: Box<dyn LanguagePlugin>) {
        let idx = self.plugins.len();
        self.by_id.insert(plugin.id().to_string(), idx);
        for ext in plugin.extensions() {
            self.by_ext.entry(ext.to_string()).or_insert(idx);
        }
        self.plugins.push(plugin);
    }

    pub fn detect(&self, path: &Path) -> anyhow::Result<&dyn LanguagePlugin> {
        let ext = path.extension()
            .and_then(|e| e.to_str())
            .unwrap_or("");

        if let Some(&idx) = self.by_ext.get(ext) {
            return Ok(self.plugins[idx].as_ref());
        }

        if let Ok(code) = std::fs::read_to_string(path) {
            for plugin in &self.plugins {
                if plugin.detect_by_content(&code) {
                    return Ok(plugin.as_ref());
                }
            }
        }

        anyhow::bail!("Language not supported for extension '{}'", ext)
    }

    pub fn all(&self) -> &[Box<dyn LanguagePlugin>] {
        &self.plugins
    }

    pub fn get(&self, id: &str) -> Option<&dyn LanguagePlugin> {
        self.by_id.get(id).map(|&idx| self.plugins[idx].as_ref())
    }
}

static REGISTRY: Lazy<LanguageRegistry> = Lazy::new(LanguageRegistry::build);

pub fn registry() -> &'static LanguageRegistry {
    &REGISTRY
}
