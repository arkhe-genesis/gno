use super::{Check, CheckResult, FileContext, Issue, IssueCategory, Severity};
use async_trait::async_trait;
use tree_sitter::{Query, QueryCursor};

pub struct UniversalConventionXCheck;

#[async_trait]
impl Check for UniversalConventionXCheck {
    fn name(&self) -> &str { "convention-x" }
    fn category(&self) -> IssueCategory { IssueCategory::ConventionX }

    async fn execute(&self, ctx: &FileContext) -> anyhow::Result<CheckResult> {
        let plugin = ctx.plugin;
        let ts_lang = match plugin.tree_sitter_language() {
            Some(l) => l,
            None => return Ok(CheckResult::default()), // Parse-only, skip
        };

        let query_str = plugin.frontier_function_query();
        if query_str.is_empty() {
             return Ok(CheckResult::default());
        }

        let query = match Query::new(&ts_lang, query_str) {
            Ok(q) => q,
            Err(_) => return Ok(CheckResult::default()),
        };

        let style = plugin.convention_x_style();
        let mut cursor = QueryCursor::new();
        let matches: Vec<_> = cursor
            .matches(&query, ctx.tree.root_node(), ctx.code.as_bytes())
            .collect();

        let issues: Vec<Issue> = matches
            .iter()
            .filter_map(|m| {
                let name_capture = m.captures.first()?;
                let name_node = name_capture.node;
                let func_name = &ctx.code[name_node.byte_range()];

                Some(Issue {
                    line: name_node.start_position().row as u32 + 1,
                    column: name_node.start_position().column as u32,
                    severity: Severity::Warning,
                    message: format!(
                        "{} '{}' recebe dados de fronteira mas não tem prefixo '{}'. {}",
                        if style.public_only { "Método público" } else { "Função" },
                        func_name,
                        style.function_prefix,
                        style.description,
                    ),
                    category: IssueCategory::ConventionX,
                })
            })
            .collect();

        let score = if issues.is_empty() { 1.0 }
            else { (1.0 - 0.15 * issues.len() as f64).max(0.0) };

        Ok(CheckResult {
            passed: issues.is_empty(),
            issues: issues.clone(),
            suggestions: issues.iter().map(|_| {
                format!("Adicione prefixo '{}' ao nome da função",
                    style.function_prefix)
            }).collect(),
            score,
        })
    }
}

pub struct UniversalSafetyCheck;

#[async_trait]
impl Check for UniversalSafetyCheck {
    fn name(&self) -> &str { "safety-patterns" }
    fn category(&self) -> IssueCategory { IssueCategory::Safety }

    async fn execute(&self, ctx: &FileContext) -> anyhow::Result<CheckResult> {
        let plugin = ctx.plugin;
        let ts_lang = match plugin.tree_sitter_language() {
            Some(l) => l,
            None => return Ok(CheckResult::default()),
        };

        let mut all_issues = Vec::new();

        for pattern in plugin.unsafe_pattern_queries() {
            let Ok(query) = Query::new(&ts_lang, pattern.query) else { continue };
            let mut cursor = QueryCursor::new();

            for m in cursor.matches(&query, ctx.tree.root_node(), ctx.code.as_bytes()) {
                let node = m.captures[0].node;
                all_issues.push(Issue {
                    line: node.start_position().row as u32 + 1,
                    column: node.start_position().column as u32,
                    severity: match pattern.severity {
                        Severity::Critical => Severity::Error,
                        other => other,
                    },
                    message: pattern.message.to_string(),
                    category: IssueCategory::Safety,
                });
            }
        }

        let penalty: f64 = all_issues.iter()
            .map(|i| match i.severity {
                Severity::Error | Severity::Critical => 0.3,
                Severity::Warning => 0.1,
                Severity::Info => 0.02,
            })
            .sum();

        Ok(CheckResult {
            passed: all_issues.is_empty(),
            issues: all_issues,
            suggestions: vec![],
            score: (1.0 - penalty).max(0.0),
        })
    }
}

pub struct UniversalDependencyCheck;

#[async_trait]
impl Check for UniversalDependencyCheck {
    fn name(&self) -> &str { "dependency-provenance" }
    fn category(&self) -> IssueCategory { IssueCategory::Dependency }

    async fn execute(&self, ctx: &FileContext) -> anyhow::Result<CheckResult> {
        let plugin = ctx.plugin;
        let ts_lang = match plugin.tree_sitter_language() {
            Some(l) => l,
            None => return Ok(CheckResult::default()),
        };

        let query_str = plugin.import_query();
        if query_str.is_empty() {
             return Ok(CheckResult::default());
        }

        let query = match Query::new(&ts_lang, query_str) {
            Ok(q) => q,
            Err(_) => return Ok(CheckResult::default()),
        };

        let mut cursor = QueryCursor::new();
        let matches: Vec<_> = cursor
            .matches(&query, ctx.tree.root_node(), ctx.code.as_bytes())
            .collect();

        let mut imports = Vec::new();
        for m in &matches {
            for capture in m.captures {
                let text = &ctx.code[capture.node.byte_range()];
                if let Some(mut import) = plugin.extract_package_from_match(ctx.code.as_ref(), text) {
                    import.line = capture.node.start_position().row as u32 + 1;
                    imports.push(import);
                }
            }
        }

        imports.sort_by(|a, b| a.package.cmp(&b.package));
        imports.dedup_by(|a, b| a.package == b.package);

        let mut issues = Vec::new();
        for imp in &imports {
            if is_likely_hallucinated(&imp.package, plugin.id()) {
                issues.push(Issue {
                    line: imp.line,
                    column: 0,
                    severity: Severity::Error,
                    message: format!(
                        "Dependência '{}' não verificada — possível alucinação de IA (slopsquatting)",
                        imp.package
                    ),
                    category: IssueCategory::Dependency,
                });
            }
            if imp.is_wildcard {
                issues.push(Issue {
                    line: imp.line,
                    column: 0,
                    severity: Severity::Warning,
                    message: format!(
                        "Import wildcard '{}' — impede análise estática de dependências",
                        imp.full_path
                    ),
                    category: IssueCategory::Dependency,
                });
            }
        }

        let score = if imports.is_empty() { 1.0 }
            else { 1.0 - (issues.len() as f64 / (imports.len() as f64).max(1.0)) };

        Ok(CheckResult {
            passed: issues.is_empty(),
            issues,
            suggestions: vec![],
            score,
        })
    }
}

fn is_likely_hallucinated(name: &str, lang: &str) -> bool {
    let generic_suffixes = [
        "-utils", "-helpers", "-core", "-tools", "-lib",
        "-common", "-shared", "-base",
    ];
    let is_generic = generic_suffixes.iter().any(|s| name.ends_with(s));

    let too_perfect = match lang {
        "python" => name.starts_with("python-") && name.len() > 8,
        "javascript" => (name.ends_with("-js") || name.starts_with("js-")) && name.len() > 5,
        "ruby" => name.starts_with("ruby-") && name.len() > 6,
        "php" => name.starts_with("php-") && name.len() > 5,
        "go" => name.starts_with("go-") && name.len() > 4,
        "java" => name.ends_with("-java") && name.len() > 6,
        _ => false,
    };

    let too_short = name.len() < 3;
    let exactly_what_llm_would_guess = matches!(name,
        "fast-api" | "fastapi" | "req" | "requests-util" | "httpx"
        | "axios-utils" | "react-hooks" | "vue-composables"
    );

    is_generic && (too_perfect || too_short || exactly_what_llm_would_guess)
}
