use crate::registry::{AnalysisTier, ConventionXStyle, ExtractedImport, LanguagePlugin, UnsafePattern};
use crate::checks::Severity;

pub struct RustPlugin;

impl LanguagePlugin for RustPlugin {
    fn id(&self) -> &str { "rust" }
    fn display_name(&self) -> &str { "Rust" }
    fn tier(&self) -> AnalysisTier { AnalysisTier::CompileTime }
    fn extensions(&self) -> &[&str] { &["rs"] }

    fn tree_sitter_language(&self) -> Option<tree_sitter::Language> {
        Some(tree_sitter_rust::language())
    }

    fn convention_x_style(&self) -> ConventionXStyle {
        ConventionXStyle {
            function_prefix: "x_",
            param_prefix: "x_",
            struct_suffix: "_x",
            public_only: false,
            description: "Funções que recebem String, Vec, &[u8], HashMap, \
                          ou tipos de crates externas devem ter prefixo 'x_'",
        }
    }

    fn import_query(&self) -> &str {
        r#"
        (use_declaration
            argument: (scoped_use_path
                path: (identifier) @crate_name
            )
        ) @use_decl

        (use_declaration
            argument: (use_as_clause
                path: (scoped_use_path
                    path: (identifier) @crate_name
                )
            )
        ) @use_as_decl

        (extern_crate_declaration
            name: (identifier) @crate_name
        ) @extern_crate
        "#
    }

    fn frontier_function_query(&self) -> &str {
        r#"
        (function_item
            name: (identifier) @fn_name
            visibility: (public) @pub
            parameters: (parameters
                (parameter
                    type: (_)
                ) @param_type
            )
            (#not-match? @fn_name "^x_")
            (#match? @param_type "String|Vec<|&\\[u8\\]|HashMap<|PathBuf|&str")
        )
        "#
    }

    fn unsafe_pattern_queries(&self) -> &[UnsafePattern] {
        &RUST_UNSAFE_PATTERNS
    }

    fn extract_package_from_match(
        &self,
        _code: &str,
        capture_text: &str,
    ) -> Option<ExtractedImport> {
        let package = capture_text.split("::").next()?.to_string();
        Some(ExtractedImport {
            package,
            full_path: capture_text.to_string(),
            line: 0,
            is_wildcard: false,
        })
    }
}

static RUST_UNSAFE_PATTERNS: &[UnsafePattern] = &[
    UnsafePattern {
        id: "rust-unwrap",
        query: r#"
            (call_expression
                function: (field_expression
                    field: (field_identifier) @method
                    (#eq? @method "unwrap")
                )
            )
        "#,
        message: ".unwrap() pode causar panic em produção. Use ? ou .unwrap_or_default().",
        severity: Severity::Error,
        auto_fixable: true,
    },
    UnsafePattern {
        id: "rust-expect",
        query: r#"
            (call_expression
                function: (field_expression
                    field: (field_identifier) @method
                    (#eq? @method "expect")
                )
            )
        "#,
        message: ".expect() causa panic com mensagem customizada. Prefira tratamento de erro explícito.",
        severity: Severity::Warning,
        auto_fixable: false,
    },
    UnsafePattern {
        id: "rust-unsafe-block",
        query: r#"(unsafe_block) @block"#,
        message: "Bloco unsafe detectado. Requer auditoria de segurança manual.",
        severity: Severity::Critical,
        auto_fixable: false,
    },
];
