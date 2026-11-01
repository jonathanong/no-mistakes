use super::super::{unique_closed_collection, RegistryKind};

const SOURCE: &str = r#"
const READ = new Map([["a", "users"]]);
const key = "a";
READ.get(key);
const MEMBER = new Map([["a", "users"]]);
void MEMBER.a;
const FOR_OF = new Map([["a", "users"]]);
for (const item of FOR_OF) {
  void item;
}
const SPREAD_ONLY = new Map([["a", "users"]]);
function take(...args: unknown[]) {}
take(...SPREAD_ONLY);
const OBJECT_SPREAD = new Map([["a", "users"]]);
const boxed = { ...OBJECT_SPREAD };
void boxed;
const OPT = new Map([["a", "users"]]);
void OPT?.get(key);
void OPT?.[key];
const OPT_SPREAD = new Map([["a", "users"]]);
function takeOpt(...args: unknown[]) {}
takeOpt?.(...OPT_SPREAD);
const FOR_IN = new Map([["a", "users"]]);
for (const item in FOR_IN) {
  void item;
}
const FROZEN = new Map([["a", "users"]]);
Object.freeze(FROZEN);
const PASSED_READ = new Map([["a", "users"]]);
const reader = {
  get(value: Map<string, string>) {
    return value;
  },
};
reader.get(PASSED_READ);
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
const EXPORTED = new Map([["a", "users"]]);
export { EXPORTED };
const WRAPPER = new Map([["a", "users"]]);
const box = { WRAPPER };
box.WRAPPER.set("b", "t");
const ARRAY_ALIAS = new Map([["a", "users"]]);
const [zz] = [ARRAY_ALIAS];
zz.set("b", "t");
export const EXPORTED_CONST = new Map([["a", "users"]]);
const ARROW = new Map([["a", "users"]]);
export const getArrow = () => ARROW;
const NESTED_ARG = new Map([["a", "users"]]);
function sink(value: object) {
  void value;
}
sink({ m: NESTED_ARG });
const DEFAULT_REG = new Map([["a", "users"]]);
export default DEFAULT_REG;
void leak;
void take;
void alias;
void sink;
"#;

#[test]
fn escaped_registries_stop_proving_closed_domains() {
    let flow = super::parse_flow(SOURCE, true);
    for name in [
        "READ",
        "MEMBER",
        "FOR_OF",
        "SPREAD_ONLY",
        "OBJECT_SPREAD",
        "OPT",
        "OPT_SPREAD",
    ] {
        assert!(
            unique_closed_collection(&flow, name, RegistryKind::Map),
            "{name}"
        );
    }
    for name in [
        "FROZEN",
        "PASSED_READ",
        "ALIAS_MAP",
        "PASSED",
        "RETURNED",
        "EXPORTED",
        "WRAPPER",
        "ARRAY_ALIAS",
        "EXPORTED_CONST",
        "ARROW",
        "NESTED_ARG",
        "DEFAULT_REG",
        "FOR_IN",
    ] {
        assert!(
            !unique_closed_collection(&flow, name, RegistryKind::Map),
            "{name}"
        );
    }
    let skipped = super::parse_flow(SOURCE, false);
    assert!(skipped.registry_mutations.is_empty());
}
