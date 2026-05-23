#[derive(Debug, Clone, PartialEq)]
pub struct EnqueueCall {
    /// Local binding identifier used (e.g. `emailsQueue`).
    pub binding: String,
    /// Job name literal if present.
    pub job: Option<String>,
    pub line: u32,
}

/// A `new Worker('queueName', handler)` declaration.
#[derive(Debug, Clone, PartialEq)]
pub struct WorkerDeclaration {
    /// Queue name string literal (e.g. `"emails"`), if present.
    pub queue_name: Option<String>,
    /// Import specifier of the processors namespace (`import * as processors from '...'`).
    pub processors_specifier: Option<String>,
    pub line: u32,
}

/// All queue-related usage patterns extracted from a source file.
#[derive(Debug, Default, Clone)]
pub struct QueueUsage {
    /// `(local_binding, import_specifier)` pairs for named imports.
    pub imports: Vec<(String, String)>,
    pub enqueue_calls: Vec<EnqueueCall>,
    pub worker_declarations: Vec<WorkerDeclaration>,
}

/// Scan `source` for queue usage patterns (enqueue calls and worker declarations).
///
/// Uses GlideMQ / BullMQ conventions:
/// - Enqueue: `<binding>.add('jobName', data)` or `<binding>.addBulk([{ name: 'jobName' }])`
/// - Worker:  `new Worker('queueName', handler)` where `handler` dispatches via
///   `processors[job.name]` from a namespace import `import * as processors from '...'`.
pub fn extract_queue_usage(source: &str) -> QueueUsage {
    let allocator = Allocator::default();
    let source_type = SourceType::ts();
    let ret = Parser::new(&allocator, source, source_type).parse();
    extract_queue_usage_from_program(&ret.program, source)
}

pub fn extract_queue_usage_from_program<'a>(program: &Program<'a>, source: &str) -> QueueUsage {
    let mut usage = QueueUsage::default();

    // Pass 1: collect named imports and namespace imports.
    let mut namespace_imports: HashMap<String, String> = HashMap::new(); // local → specifier
    for stmt in &program.body {
        if let Statement::ImportDeclaration(import_decl) = stmt {
            let src = import_decl.source.value.as_str();
            if let Some(specifiers) = &import_decl.specifiers {
                for spec in specifiers {
                    match spec {
                        ImportDeclarationSpecifier::ImportSpecifier(s) => {
                            let local = s.local.name.as_str().to_string();
                            usage.imports.push((local, src.to_string()));
                        }
                        ImportDeclarationSpecifier::ImportNamespaceSpecifier(s) => {
                            namespace_imports
                                .insert(s.local.name.as_str().to_string(), src.to_string());
                        }
                        _ => {}
                    }
                }
            }
        }
    }

    // Also include bindings from default imports (less common for queues, but handle for robustness).
    for stmt in &program.body {
        if let Statement::ImportDeclaration(import_decl) = stmt {
            let src = import_decl.source.value.as_str();
            if let Some(specifiers) = &import_decl.specifiers {
                for spec in specifiers {
                    if let ImportDeclarationSpecifier::ImportDefaultSpecifier(s) = spec {
                        let local = s.local.name.as_str().to_string();
                        usage.imports.push((local, src.to_string()));
                    }
                }
            }
        }
    }

    // Pass 2: scan statements for enqueue calls and Worker constructors.
    for stmt in &program.body {
        scan_stmt(stmt, source, &namespace_imports, &mut usage);
    }

    usage
}
