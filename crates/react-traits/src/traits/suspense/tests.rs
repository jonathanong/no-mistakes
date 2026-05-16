use super::detect_uses_suspense;
use no_mistakes_core::ast;

fn check(source: &str) -> bool {
    let path = std::path::Path::new("test.tsx");
    let span = oxc_span::Span::new(0, source.len() as u32);
    ast::with_program(path, source, |program, _| {
        detect_uses_suspense(program, span)
    })
    .unwrap()
}

#[test]
fn detects_suspense_jsx() {
    assert!(check(
        "export default function App() { return <Suspense fallback={null}><div/></Suspense>; }"
    ));
}

#[test]
fn detects_next_dynamic_component_rendered() {
    // dynamic() creates a lazily-loaded component; rendering it in JSX triggers uses_suspense
    assert!(check(
        "const Lazy = dynamic(() => import('./Foo')); export default function App() { return <Lazy/>; }"
    ));
}

#[test]
fn dynamic_import_without_render_not_suspense() {
    // importing next/dynamic without rendering the resulting component = no suspense (Chper)
    assert!(!check(
        "import dynamic from 'next/dynamic'; export default function App() { return <div/>; }"
    ));
}

#[test]
fn detects_react_lazy_component_rendered() {
    // React.lazy() component rendered in JSX within span triggers uses_suspense
    assert!(check(
        "const Lazy = React.lazy(() => import('./Foo')); export default function App() { return <Lazy/>; }"
    ));
}

#[test]
fn dynamic_component_outside_span_not_detected() {
    // dynamic-named JSX element outside span should not trigger suspense
    let source = "const Lazy = dynamic(() => import('./Foo')); export default function App() { return <Lazy/>; }";
    let path = std::path::Path::new("test.tsx");
    let result = no_mistakes_core::ast::with_program(path, source, |program, _| {
        super::detect_uses_suspense(program, oxc_span::Span::new(0, 0))
    })
    .unwrap();
    assert!(!result);
}

#[test]
fn no_suspense() {
    assert!(!check("export default function App() { return <div/>; }"));
}

#[test]
fn detects_react_suspense_member() {
    assert!(check(
        "export default function App() { return <React.Suspense fallback={null}><div/></React.Suspense>; }"
    ));
}

#[test]
fn export_default_dynamic_is_suspense() {
    // `export default dynamic(...)` — component itself is a dynamic wrapper
    assert!(check("export default dynamic(() => import('./Heavy'));"));
}

#[test]
fn export_default_lazy_is_suspense() {
    // `export default lazy(...)` — component itself is a lazy wrapper
    assert!(check("export default lazy(() => import('./Heavy'));"));
}

#[test]
fn export_const_dynamic_component_is_suspense() {
    // `export const Lazy = dynamic(...)` — named export dynamic wrapper is suspense
    assert!(check(
        "export const Lazy = dynamic(() => import('./Heavy'));"
    ));
}

#[test]
fn named_dynamic_component_rendered_from_named_export() {
    // `export const Lazy = dynamic(...)` then render `<Lazy/>` — suspense from rendering it
    assert!(check(
        "export const Lazy = dynamic(() => import('./Heavy')); export default function App() { return <Lazy/>; }"
    ));
}

#[test]
fn suspense_outside_span_not_detected() {
    // Span that covers nothing — visit_jsx_opening_element returns early (line 16-17).
    let source =
        "export default function App() { return <Suspense fallback={null}><div/></Suspense>; }";
    let path = std::path::Path::new("test.tsx");
    let result = no_mistakes_core::ast::with_program(path, source, |program, _| {
        super::detect_uses_suspense(program, oxc_span::Span::new(0, 0))
    })
    .unwrap();
    assert!(!result);
}
