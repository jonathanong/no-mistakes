use super::*;

#[test]
fn extract_ignores_nested_next_config_binding() {
    let source = "function build() {\n\
  const nextConfig = { cacheComponents: true }\n\
  return nextConfig\n\
}\n\
const nextConfig = {}\n\
export default nextConfig\n";
    let findings = extract(Path::new("next.config.ts"), source).unwrap();

    assert!(findings.is_empty());
}
