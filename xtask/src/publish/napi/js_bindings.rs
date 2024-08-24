use std::fmt::Write;

pub fn create_cjs_binding(local_name: &str, package_name: &str, idents: &[String]) -> String {
    let template = include_str!("bindings.js");

    let make_require_tuple = |tuple: &str| {
        format!(
            r#"
            try {{
                return require('./{local_name}.{tuple}.node')
            }} catch (e) {{
                loadErrors.push(e)
            }}
            try {{
                return require('{package_name}-{tuple}')
            }} catch (e) {{
                loadErrors.push(e)
            }}
            "#
        )
    };

    // Replace REQUIRE_TUPLE with make_require_tuple
    let mut result = template.to_string();
    let prefix = "REQUIRE_TUPLE(";
    while let Some(start) = result.find(prefix) {
        if let Some(end) = result[start..].find(')') {
            let tuple = &result[start + prefix.len()..start + end].trim_matches('\'').trim_matches('"');
            let replacement = make_require_tuple(tuple);
            result.replace_range(start..start + end + 1, &replacement);
        } else {
            break;
        }
    }

    // Replace placeholders
    result = result.replace("${localName}", local_name);
    result = result.replace("${pkgName}", package_name);

    // Generate IDENTS section
    let mut idents_section = String::new();
    for ident in idents {
        writeln!(idents_section, "module.exports.{0} = nativeBinding.{0}", ident).unwrap();
    }
    result = result.replace("IDENTS", &idents_section);

    result
}


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_create_cjs_binding() {
        let local_name = "test_local";
        let package_name = "test_package";
        let idents = vec!["func1".to_string(), "func2".to_string()];

        let result = create_cjs_binding(local_name, package_name, &idents);

        dbg!(&result);

        // Test that the local_name is correctly inserted
        assert!(result.contains(&format!("require('./{}", local_name)));

        // Test that the package_name is correctly inserted
        assert!(result.contains(&format!("require('{}-", package_name)));

        // Test that the idents are correctly exported
        for ident in &idents {
            assert!(result.contains(&format!("module.exports.{0} = nativeBinding.{0}", ident)));
        }

        // Test that the REQUIRE_TUPLE macro is expanded correctly
        assert!(result.contains(&format!("return require('./{}.darwin-x64.node')", local_name)));
        assert!(result.contains(&format!("return require('{}-darwin-x64')", package_name)));

        // Test that the placeholders are replaced
        assert!(!result.contains("${localName}"));
        assert!(!result.contains("${pkgName}"));

        // Test that the IDENTS placeholder is replaced
        assert!(!result.contains("IDENTS"));
    }

    #[test]
    fn test_create_cjs_binding_empty_idents() {
        let local_name = "empty_test";
        let package_name = "empty_package";
        let idents: Vec<String> = vec![];

        let result = create_cjs_binding(local_name, package_name, &idents);

        // Test that the function doesn't panic with empty idents
        assert!(!result.contains("module.exports."));
    }
}

