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
const LITERAL_GET = new Map([["a", "users"]]);
LITERAL_GET["get"](key);
const NON_CALLEE = new Map([["a", "users"]]);
void NON_CALLEE["set"];
const ARG_READ = new Map([["a", "users"]]);
function takeArg(value: unknown) {
  void value;
}
takeArg(ARG_READ[key]);
const OPT_GET = new Map([["a", "users"]]);
OPT_GET?.["get"](key);
const PAREN_GET = new Map([["a", "users"]]);
(PAREN_GET["get"])(key);
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
const A_SET = new Map([["a", "users"]]);
A_SET["set"]("b", "t");
const B_CLEAR = new Map([["a", "users"]]);
B_CLEAR["clear"]();
const C_DYN = new Map([["a", "users"]]);
const method = "set";
C_DYN[method]("b", "t");
const D_PUSH = ["users"];
D_PUSH["push"]("t");
const E_ADD = new Set(["users"]);
E_ADD["add"]("t");
const UNKNOWN_KEY = new Map([["a", "users"]]);
UNKNOWN_KEY["foo"]();
const NESTED_SET = new Map([["a", "users"]]);
function outer(value: unknown) {
  void value;
}
outer(NESTED_SET["set"]("b", "t"));
const OPT_SET = new Map([["a", "users"]]);
OPT_SET?.["set"]("b", "t");
const PAREN_CLEAR = new Map([["a", "users"]]);
(PAREN_CLEAR["clear"])();
const DYN_GET = new Map([["a", "users"]]);
const getName = "get";
DYN_GET[getName]("a");
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
        "LITERAL_GET",
        "NON_CALLEE",
        "ARG_READ",
        "OPT_GET",
        "PAREN_GET",
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
        "A_SET",
        "B_CLEAR",
        "C_DYN",
        "UNKNOWN_KEY",
        "NESTED_SET",
        "OPT_SET",
        "PAREN_CLEAR",
        "DYN_GET",
    ] {
        assert!(
            !unique_closed_collection(&flow, name, RegistryKind::Map),
            "{name}"
        );
    }
    assert!(
        !unique_closed_collection(&flow, "D_PUSH", RegistryKind::Array),
        "D_PUSH"
    );
    assert!(
        !unique_closed_collection(&flow, "E_ADD", RegistryKind::Set),
        "E_ADD"
    );
    for name in ["A_SET", "B_CLEAR", "C_DYN", "D_PUSH", "E_ADD"] {
        assert!(
            flow.registry_mutations
                .iter()
                .any(|mutation| mutation.name == name),
            "{name}"
        );
    }
    let skipped = super::parse_flow(SOURCE, false);
    assert!(skipped.registry_mutations.is_empty());
}
