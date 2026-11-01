use super::{unique_closed_collection, GuardKind, RegistryKind, SetMethod};
use crate::codebase::dependencies::extract::extract_import_facts_from_program_with_source_and_demands;
use oxc_allocator::Allocator;
use oxc_parser::Parser;
use oxc_span::SourceType;

#[test]
fn closed_domain_facts_reject_open_shapes() {
    let source = r#"
        declare const extra: string;
        declare const kind: string;
        declare const suffix: string;
        declare const obj: { assertWhitelistedSqlIdentifier(value: string): void };
        declare function randomUUID(...args: string[]): string;
        declare function id(): string;
        declare function assertWhitelistedSqlIdentifier(...args: unknown[]): void;

        const user = "users";
        const KEEP_OBJECT = { user: "users" };
        const KEEP_ARRAY = ["users"];
        const KEEP_MAP = new Map([["user", "users"]]);
        const KEEP_SET = new Set(["users"]);
        const KEEP_FROZEN = Object.freeze({ user: "users" });
        const KEEP_FROZEN_MAP = Object.freeze(new Map([["user", "users"]]));
        const copied = KEEP_OBJECT["user"];
        const copiedStatic = KEEP_OBJECT.user;
        const fromArray = KEEP_ARRAY[0];
        const fromMap = KEEP_MAP.get(kind);
        const goodName = "tmp_" + randomUUID().replace(/-/g, "");

        const EMPTY_OBJECT = {};
        const EMPTY_ARRAY = [];
        const EMPTY_SET = new Set([]);
        const EMPTY_MAP = new Map([]);
        const SPREAD = { ...KEEP_OBJECT, user: "users" };
        const SHORTHAND = { user };
        const METHOD = { user() { return "users"; } };
        const GETTER = { get user() { return "users"; } };
        const COMPUTED = { ["user"]: "users" };
        const NON_LITERAL = { user: extra };
        const HOLE = ["users", , "posts"];
        const ARRAY_SPREAD = ["users", ..."posts"];
        const BAD_PAIR = new Map([["user"]]);
        const FLAT_MAP = new Map(["user", "users"]);
        const WEAK = new WeakMap([["user", "users"]]);
        const FREEZE_NONE = Object.freeze();
        const FREEZE_EXTRA = Object.freeze({ user: "users" }, 1);
        const OPT_FREEZE = Object.freeze?.({ user: "users" });
        const OPT_MEMBER = KEEP_OBJECT?.user;
        const OPT_GET = KEEP_MAP?.get(kind);
        const { renamed: alias } = { renamed: "users" };

        const emptyPrefix = `${randomUUID().replaceAll("-", "")}`;
        const flags = "tmp_" + randomUUID().replace(/-/gi, "");
        const noGlobal = "tmp_" + randomUUID().replace(/-/, "");
        const badReplacement = `tmp_${randomUUID().replaceAll("-", "x")}`;
        const regexReplaceAll = `tmp_${randomUUID().replaceAll(/-/, "")}`;
        const withArg = `tmp_${randomUUID(1).replaceAll("-", "")}`;
        const otherCallee = `tmp_${id().replaceAll("-", "")}`;
        const optUuid = `tmp_${randomUUID?.().replaceAll("-", "")}`;
        const emptyAdd = "" + randomUUID().replaceAll("-", "");
        const mixed = `tmp_${randomUUID().replaceAll("-", "")}_${suffix}`;

        function shadow(ALLOWED: string) {
          if (!ALLOWED.has(ALLOWED)) throw new Error("x");
        }
        const ALLOWED = new Set(["users"]);
        const TABLES = new Map([["user", "users"]]);
        function second() {
          const TABLES = new Map([["user", "posts"]]);
        }
        function useTables(kindName: string) {
          const table = TABLES.get(kindName);
          return table;
        }
        function elseCompletes(table: string) {
          if (!KEEP_SET.has(table)) throw new Error("x");
          else table;
        }
        function thenCompletes(table: string) {
          if (!KEEP_SET.has(table)) table;
          else throw new Error("x");
        }
        function noThrow(table: string) {
          if (!KEEP_ARRAY.includes(table)) table;
        }
        function optional(table: string) {
          if (!KEEP_SET?.has(table)) throw new Error("x");
        }
        function wrongMethod(table: string) {
          if (!KEEP_SET.add(table)) throw new Error("x");
        }
        function wrongArity(table: string) {
          if (!KEEP_SET.has(table, table)) throw new Error("x");
        }
        function positive(table: string) {
          if (KEEP_SET.has(table)) throw new Error("x");
        }
        function nested(table: string) {
          {
            if (!KEEP_SET.has(table)) throw new Error("x");
          }
        }
        function good(table: string) {
          if (!KEEP_SET.has(table)) throw new Error("x");
          if (!KEEP_ARRAY.includes(table)) throw new Error("x");
          assertWhitelistedSqlIdentifier(table, KEEP_SET, "table");
        }
        function dotted(table: string) {
          obj.assertWhitelistedSqlIdentifier(table);
        }
        function spreadAssert(table: string) {
          assertWhitelistedSqlIdentifier(table, ...KEEP_ARRAY);
        }
        function optionalAssert(table: string) {
          assertWhitelistedSqlIdentifier?.(table);
        }
        void shadow;
        void second;
        void useTables;
        void elseCompletes;
        void thenCompletes;
        void noThrow;
        void optional;
        void wrongMethod;
        void wrongArity;
        void positive;
        void nested;
        void good;
        void dotted;
        void spreadAssert;
        void optionalAssert;
        void alias;
        void EMPTY_OBJECT;
        void EMPTY_ARRAY;
        void EMPTY_SET;
        void EMPTY_MAP;
        void SPREAD;
        void SHORTHAND;
        void METHOD;
        void GETTER;
        void COMPUTED;
        void NON_LITERAL;
        void HOLE;
        void ARRAY_SPREAD;
        void BAD_PAIR;
        void FLAT_MAP;
        void WEAK;
        void FREEZE_NONE;
        void FREEZE_EXTRA;
        void OPT_FREEZE;
        void OPT_MEMBER;
        void OPT_GET;
        void emptyPrefix;
        void flags;
        void noGlobal;
        void badReplacement;
        void regexReplaceAll;
        void withArg;
        void otherCallee;
        void optUuid;
        void emptyAdd;
        void mixed;
        void copied;
        void copiedStatic;
        void fromArray;
        void fromMap;
        void goodName;
    "#;
    let flow = parse_flow(source, true);
    let mut names: Vec<_> = flow
        .closed_string_registries
        .iter()
        .map(|registry| registry.name.as_str())
        .collect();
    names.sort_unstable();
    assert_eq!(
        names,
        [
            "ALLOWED",
            "KEEP_ARRAY",
            "KEEP_FROZEN",
            "KEEP_FROZEN_MAP",
            "KEEP_MAP",
            "KEEP_OBJECT",
            "KEEP_SET",
            "TABLES",
            "TABLES",
        ]
    );
    assert!(unique_closed_collection(
        &flow,
        "KEEP_SET",
        RegistryKind::Set
    ));
    assert!(unique_closed_collection(
        &flow,
        "KEEP_OBJECT",
        RegistryKind::Object
    ));
    assert!(unique_closed_collection(
        &flow,
        "KEEP_ARRAY",
        RegistryKind::Array
    ));
    assert!(unique_closed_collection(
        &flow,
        "KEEP_MAP",
        RegistryKind::Map
    ));
    assert!(!unique_closed_collection(
        &flow,
        "KEEP_SET",
        RegistryKind::Array
    ));
    assert!(!unique_closed_collection(
        &flow,
        "ALLOWED",
        RegistryKind::Set
    ));
    assert!(!unique_closed_collection(
        &flow,
        "TABLES",
        RegistryKind::Map
    ));
    assert!(!unique_closed_collection(
        &flow,
        "missing",
        RegistryKind::Set
    ));
    let mut bounded: Vec<_> = flow
        .bounded_bindings
        .iter()
        .map(|binding| binding.name.as_str())
        .collect();
    bounded.sort_unstable();
    assert_eq!(
        bounded,
        ["copied", "copiedStatic", "fromArray", "fromMap", "goodName"]
    );
    assert_eq!(flow.finite_identifier_guards.len(), 6);
    assert!(flow.finite_identifier_guards.iter().any(|guard| {
        guard.name == "ALLOWED"
            && matches!(
                guard.kind,
                GuardKind::ClosedSet {
                    ref collection,
                    method: SetMethod::Has,
                } if collection == "ALLOWED"
            )
    }));
    assert!(flow.finite_identifier_guards.iter().any(|guard| {
        matches!(
            guard.kind,
            GuardKind::ClosedSet {
                ref collection,
                method: SetMethod::Includes,
            } if collection == "KEEP_ARRAY"
        )
    }));
    assert!(flow.finite_identifier_guards.iter().any(|guard| {
        matches!(
            &guard.kind,
            GuardKind::Assertion { callee, argument: 0 } if callee == "assertWhitelistedSqlIdentifier"
        )
    }));
    let skipped = parse_flow(source, false);
    assert!(skipped.closed_string_registries.is_empty());
    assert!(skipped.bounded_bindings.is_empty());
    assert!(skipped.finite_identifier_guards.is_empty());
}

fn parse_flow(
    source: &str,
    collect_value_flow: bool,
) -> crate::codebase::dependencies::extract::ValueFlowFacts {
    let allocator = Allocator::default();
    let parsed = Parser::new(&allocator, source, SourceType::ts()).parse();
    assert!(parsed.diagnostics.is_empty(), "{:?}", parsed.diagnostics);
    extract_import_facts_from_program_with_source_and_demands(
        &parsed.program,
        source,
        false,
        false,
        collect_value_flow,
    )
    .value_flow
}
