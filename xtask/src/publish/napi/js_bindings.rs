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
            let tuple = &result[start + prefix.len()..start + end]
                .trim_matches('\'')
                .trim_matches('"');
            let replacement = make_require_tuple(tuple);
            result.replace_range(start..start + end + 1, &replacement);
        } else {
            break;
        }
    }

    // Replace placeholders
    result = result.replace("{localName}", local_name);
    result = result.replace("{pkgName}", package_name);

    // Generate IDENTS section
    let mut idents_section = String::new();
    for ident in idents {
        writeln!(
            idents_section,
            "module.exports.{0} = nativeBinding.{0}",
            ident
        )
        .unwrap();
    }
    result = result.replace("IDENTS", &idents_section);

    result
}
