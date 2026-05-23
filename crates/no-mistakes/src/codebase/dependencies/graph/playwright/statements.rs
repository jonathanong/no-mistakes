
/// Extract URL string literals navigated to in a Playwright test file.
///
/// Recognises:
/// - `page.goto('<url>')`
/// - ``page.goto(`/users/${id}`)`` (normalizes interpolations to `:param`)
/// - `page.click('a[href="<url>"]')` / `page.click("a[href='<url>']")`
/// - `navigateTo(page, '<url>')` / `navigateTo('<url>')`
/// - `expect(page).toHaveURL('<url>')` and `page.waitForURL('<url>')`
pub fn extract_playwright_urls(source: &str) -> Vec<String> {
    let allocator = Allocator::default();
    let source_type = SourceType::tsx();
    let ret = Parser::new(&allocator, source, source_type).parse();

    let mut urls = Vec::new();

    for stmt in &ret.program.body {
        collect_urls_from_stmt(stmt, &mut urls);
    }

    urls.sort();
    urls.dedup();
    urls
}

fn collect_urls_from_stmt(stmt: &Statement, urls: &mut Vec<String>) {
    match stmt {
        Statement::ExpressionStatement(s) => collect_urls_from_expr(&s.expression, urls),
        Statement::VariableDeclaration(v) => {
            for decl in &v.declarations {
                if let Some(init) = &decl.init {
                    collect_urls_from_expr(init, urls);
                }
            }
        }
        Statement::ReturnStatement(r) => {
            if let Some(e) = &r.argument {
                collect_urls_from_expr(e, urls);
            }
        }
        Statement::BlockStatement(b) => {
            for s in &b.body {
                collect_urls_from_stmt(s, urls);
            }
        }
        Statement::FunctionDeclaration(f) => {
            collect_urls_from_body(f.body.as_deref(), urls);
        }
        Statement::IfStatement(i) => {
            collect_urls_from_expr(&i.test, urls);
            collect_urls_from_stmt(&i.consequent, urls);
            if let Some(alt) = &i.alternate {
                collect_urls_from_stmt(alt, urls);
            }
        }
        Statement::TryStatement(t) => {
            collect_urls_from_stmts(&t.block.body, urls);
            if let Some(handler) = &t.handler {
                collect_urls_from_stmts(&handler.body.body, urls);
            }
            if let Some(finalizer) = &t.finalizer {
                collect_urls_from_stmts(&finalizer.body, urls);
            }
        }
        Statement::WhileStatement(w) => {
            collect_urls_from_expr(&w.test, urls);
            collect_urls_from_stmt(&w.body, urls);
        }
        Statement::DoWhileStatement(d) => {
            collect_urls_from_stmt(&d.body, urls);
            collect_urls_from_expr(&d.test, urls);
        }
        Statement::ForStatement(f) => collect_urls_from_for_stmt(f, urls),
        Statement::ForInStatement(f) => {
            collect_urls_from_expr(&f.right, urls);
            collect_urls_from_stmt(&f.body, urls);
        }
        Statement::ForOfStatement(f) => {
            collect_urls_from_expr(&f.right, urls);
            collect_urls_from_stmt(&f.body, urls);
        }
        Statement::SwitchStatement(s) => {
            collect_urls_from_expr(&s.discriminant, urls);
            for case in &s.cases {
                if let Some(test) = &case.test {
                    collect_urls_from_expr(test, urls);
                }
                for stmt in &case.consequent {
                    collect_urls_from_stmt(stmt, urls);
                }
            }
        }
        _ => {}
    }
}

