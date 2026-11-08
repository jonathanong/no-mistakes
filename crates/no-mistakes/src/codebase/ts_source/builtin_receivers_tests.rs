use super::collect;
use oxc_allocator::Allocator;
use oxc_parser::Parser;
use oxc_span::SourceType;

/// Source prefixes (up to the first argument list) of calls with a proven receiver.
fn proven(source: &str) -> Vec<String> {
    let allocator = Allocator::default();
    let program = Parser::new(&allocator, source, SourceType::mjs().with_typescript(true))
        .parse()
        .program;
    let mut offsets: Vec<u32> = collect(&program).into_iter().collect();
    offsets.sort_unstable();
    offsets
        .into_iter()
        .map(|at| {
            let rest = &source[at as usize..];
            let end = rest[1..].find('(').map_or(rest.len(), |i| i + 1);
            rest[..end].to_string()
        })
        .collect()
}

#[test]
fn literals_constructors_and_factories_are_built_in() {
    let found = proven(
        "'a'.trim(); `x`.trim(); /a/.test('b'); [1].map(f); ({ a: 1 }).hasOwnProperty('a');\n\
         new Map().get(1); Array.from([]).map(f); Object.keys(o).map(f); JSON.stringify(1).trim();\n\
         String(1).trim(); (x ? 'a' : 'b').trim();",
    );
    assert_eq!(found.len(), 11, "{found:?}");
}

#[test]
fn unknown_or_colliding_receivers_are_not_built_in() {
    let found = proven(
        "const sinks: readonly Sink[] = []; const sink = sinks[0]; sink.push(1);\n\
         ({ push() {} }).push(1); ({ ...o }).push(1); ({ [k]: 1 }).push(1);\n\
         const Map = make(); new Map().get(1); x.push(1); a.b.c(); get().push(1);\n\
         (await p).push(1); (y as Sink).push(1);",
    );
    assert!(found.is_empty(), "{found:?}");
}

#[test]
fn declarations_decide_named_receivers() {
    let found = proven(
        "const items = []; const text: string = ''; let loose = []; const copy = items.map(f);\n\
         function use(a: string, b: Map<string, Sink> | undefined, c: Sink, d = 1, [e]: string[]) {\n\
           items.push(1); text.trim(); loose.push(1); copy.at(0); a.trim(); b.set('k', c); c.push(1);\n\
           d.toFixed(); e.trim(); (items as string[]).push(1); items!.push(1);\n\
         }\n\
         { const text: Sink = make(); text.trim(); }",
    );
    assert_eq!(
        found,
        [
            "items.map",
            "items.push",
            "copy.at",
            "a.trim",
            "b.set",
            "(items as string[]).push",
            "items!.push"
        ]
    );
}

#[test]
fn async_hooks_storage_import_is_built_in() {
    let found = proven(
        "import { AsyncLocalStorage } from 'node:async_hooks';\n\
         import { AsyncLocalStorage as Other } from './local';\n\
         import fallback from 'x';\n\
         const a = new AsyncLocalStorage(); const b = new Other();\n\
         a.run(1); b.run(1);",
    );
    assert_eq!(found, ["a.run"]);
}

#[test]
fn deeply_chained_initializers_stop_at_the_depth_bound() {
    let mut source = String::from("const v0 = [];\n");
    for n in 1..12 {
        source.push_str(&format!("const v{n} = v{}.slice();\n", n - 1));
    }
    source.push_str("v11.push(1);");
    assert!(!proven(&source).contains(&"v11.push".to_string()));
}
