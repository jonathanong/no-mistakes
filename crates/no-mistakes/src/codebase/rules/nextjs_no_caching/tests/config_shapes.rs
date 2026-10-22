use super::extract;
use std::path::Path;

#[test]
fn extract_handles_defensive_next_config_shapes() {
    let source = "const key = 'cacheLife'\n\
const nextConfig = { cacheHandlers: {} }\n\
export default ({ ...nextConfig, [key]: {}, cacheComponents: 'yes' })\n";
    let findings = extract(Path::new("next.config.ts"), source).unwrap();

    assert!(findings.is_empty());
}

#[test]
fn extract_reports_wrapped_commonjs_next_config_identifier() {
    let source = "const nextConfig = {\n\
  cacheLife: {},\n\
}\n\
module.exports = (withConfig(nextConfig) satisfies NextConfig)\n";
    let findings = extract(Path::new("next.config.ts"), source).unwrap();

    assert_eq!(findings.len(), 1);
    assert!(findings
        .iter()
        .any(|finding| finding.message.contains("cacheLife")));
}

#[test]
fn extract_reports_commonjs_next_config_identifier_and_wrappers() {
    let source = "const nextConfig = { cacheHandlers: {} }\n\
module.exports = nextConfig\n\
module.exports = (nextConfig as NextConfig)\n";
    let findings = extract(Path::new("next.config.ts"), source).unwrap();

    assert_eq!(findings.len(), 1);
    assert!(findings
        .iter()
        .any(|finding| finding.message.contains("cacheHandlers")));
}

#[test]
fn extract_reports_wrapped_commonjs_next_config_objects() {
    let source = "module.exports = ({ cacheHandlers: {} })\n\
module.exports = ({ cacheLife: {} } as NextConfig)\n";
    let findings = extract(Path::new("next.config.ts"), source).unwrap();

    assert_eq!(findings.len(), 2);
    assert!(findings
        .iter()
        .any(|finding| finding.message.contains("cacheHandlers")));
    assert!(findings
        .iter()
        .any(|finding| finding.message.contains("cacheLife")));
}

#[test]
fn extract_handles_defensive_assignment_and_argument_shapes() {
    let source = "export default withConfig('name', { cacheComponents: true });\n\
({ value } = source)\n";
    let findings = extract(Path::new("next.config.ts"), source).unwrap();

    assert_eq!(findings.len(), 1);
    assert!(findings
        .iter()
        .any(|finding| finding.message.contains("cacheComponents")));
}

#[test]
fn extract_ignores_non_module_exports_assignment_in_next_config() {
    let source = "exports.config = { cacheComponents: true }\n";
    let findings = extract(Path::new("next.config.js"), source).unwrap();

    assert!(findings.is_empty());
}

#[test]
fn extract_ignores_default_config_object_outside_next_config_files() {
    let source = "export default {\n\
  cacheComponents: true,\n\
  cacheLife: {},\n\
}\n";
    let findings = extract(Path::new("app/config.ts"), source).unwrap();

    assert!(findings.is_empty());
}

#[test]
fn extract_ignores_commonjs_config_object_outside_next_config_files() {
    let source = "module.exports = {\n\
  cacheComponents: true,\n\
  cacheHandlers: {},\n\
}\n";
    let findings = extract(Path::new("lib/config.cjs"), source).unwrap();

    assert!(findings.is_empty());
}
