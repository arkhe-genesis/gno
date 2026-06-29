use crate::registry::{AnalysisTier, ConventionXStyle, ExtractedImport, LanguagePlugin, UnsafePattern};
use crate::checks::Severity;

pub struct PythonPlugin;

impl LanguagePlugin for PythonPlugin {
    fn id(&self) -> &str { "python" }
    fn display_name(&self) -> &str { "Python 3" }
    fn tier(&self) -> AnalysisTier { AnalysisTier::Runtime }
    fn extensions(&self) -> &[&str] { &["py", "pyi", "pyw"] }

    fn tree_sitter_language(&self) -> Option<tree_sitter::Language> {
        Some(tree_sitter_python::language())
    }

    fn convention_x_style(&self) -> ConventionXStyle {
        ConventionXStyle {
            function_prefix: "x_",
            param_prefix: "x_",
            struct_suffix: "_x",
            public_only: false,
            description: "Funções que recebem str, list, dict, Any, \
                          ou dados de request/response devem ter prefixo 'x_'",
        }
    }

    fn import_query(&self) -> &str {
        r#"
        (import_statement
            name: (dotted_name) @module_name
        )
        (import_from_statement
            module_name: (dotted_name) @module_name
            name: (aliased_import) @aliased
        )
        (import_from_statement
            module_name: (dotted_name) @module_name
            wildcard: (wildcard_import) @wildcard
        )
        "#
    }

    fn frontier_function_query(&self) -> &str {
        r#"
        (function_definition
            name: (identifier) @fn_name
            parameters: (parameters
                (parameter
                    type: (type) @ptype
                )
            )
            (#not-match? @fn_name "^x_")
            (#not-match? @fn_name "^_")
            (#match? @ptype "str|list|dict|Any|Request|Response|JSONObject")
        )
        "#
    }

    fn unsafe_pattern_queries(&self) -> &[UnsafePattern] {
        &PYTHON_UNSAFE_PATTERNS
    }

    fn extract_package_from_match(
        &self,
        _code: &str,
        capture_text: &str,
    ) -> Option<ExtractedImport> {
        let package = capture_text.split('.').next()?.to_string();
        let stdlib = [
            "os", "sys", "json", "re", "math", "time", "datetime",
            "collections", "itertools", "functools", "pathlib",
            "typing", "dataclasses", "abc", "io", "copy",
        ];
        if stdlib.contains(&package.as_str()) { return None; }
        Some(ExtractedImport {
            package,
            full_path: capture_text.to_string(),
            line: 0,
            is_wildcard: false,
        })
    }
}

static PYTHON_UNSAFE_PATTERNS: &[UnsafePattern] = &[
    UnsafePattern {
        id: "python-eval",
        query: r#"
            (call
                function: (identifier) @fn
                (#eq? @fn "eval")
            )
        "#,
        message: "eval() executa código arbitrário.",
        severity: Severity::Critical,
        auto_fixable: false,
    },
];
