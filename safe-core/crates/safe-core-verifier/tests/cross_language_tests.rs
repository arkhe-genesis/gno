macro_rules! test_unsafe_pattern {
    ($test_name:ident, $lang_feature:expr, $lang_id:expr, $code:expr, $expected_pattern_id:expr) => {
        #[test]
        fn $test_name() {
            let reg = safe_core_verifier::registry::registry();
            let plugin = reg.get($lang_id).expect(&format!("lang {} not registered", $lang_id));
            let ts_lang = plugin.tree_sitter_language()
                .expect(&format!("{} has no tree-sitter support", $lang_id));

            let mut parser = tree_sitter::Parser::new();
            parser.set_language(&ts_lang).unwrap();
            let tree = parser.parse($code, None).unwrap();

            let found: Vec<_> = plugin.unsafe_pattern_queries()
                .iter()
                .filter(|p| {
                    let Ok(query) = tree_sitter::Query::new(&ts_lang, p.query) else { return false };
                    let mut cursor = tree_sitter::QueryCursor::new();
                    cursor.matches(&query, tree.root_node(), $code.as_bytes()).count() > 0
                })
                .map(|p| p.id)
                .collect();

            assert!(found.contains(&$expected_pattern_id),
                "Expected pattern '{}' not found in {}. Found: {:?}",
                $expected_pattern_id, $lang_id, found);
        }
    };
}

test_unsafe_pattern!(
    unwrap_unsafe_in_rust, "lang-rust", "rust",
    r#"fn process(data: &str) -> usize { data.parse::<usize>().unwrap() }"#,
    "rust-unwrap"
);
