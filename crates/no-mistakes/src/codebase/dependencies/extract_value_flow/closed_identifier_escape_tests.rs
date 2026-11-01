use super::super::{unique_closed_collection, RegistryKind};

const SOURCE: &str = r#"
const READ = new Map([["a", "users"]]);
const key = "a";
READ.get(key);
const holder = {
  get(value: Map<string, string>) {
    return value;
  },
};
holder.get(READ);
const FROZEN = new Map([["a", "users"]]);
Object.freeze(FROZEN);
const SPREAD_ONLY = new Map([["a", "users"]]);
function take(...args: unknown[]) {}
take(...SPREAD_ONLY);
const ALIAS_MAP = new Map([["a", "users"]]);
const alias = ALIAS_MAP;
const PASSED = new Map([["a", "users"]]);
function fill(m: Map<string, string>) {
  m.set("z", "t");
}
fill(PASSED);
const RETURNED = new Map([["a", "users"]]);
function leak() {
  return RETURNED;
}
function bare() {
  return;
}
function other() {
  return key;
}
const EXPORTED = new Map([["a", "users"]]);
export { EXPORTED };
void leak;
void bare;
void other;
void alias;
void take;
"#;

#[test]
fn escaped_registries_stop_proving_closed_domains() {
    let flow = super::parse_flow(SOURCE, true);
    for name in ["ALIAS_MAP", "PASSED", "RETURNED", "EXPORTED"] {
        assert!(
            !unique_closed_collection(&flow, name, RegistryKind::Map),
            "{name}"
        );
    }
    for name in ["READ", "FROZEN", "SPREAD_ONLY"] {
        assert!(
            unique_closed_collection(&flow, name, RegistryKind::Map),
            "{name}"
        );
    }
    let skipped = super::parse_flow(SOURCE, false);
    assert!(skipped.registry_mutations.is_empty());
}
