impl Default for TsFactContext {
    fn default() -> Self {
        Self {
            root: PathBuf::new(),
            argument_projection: super::super::ArgumentProjection::default(),
            backend_route_extractors: Vec::new(),
            queue_factory_specifier: None,
            queue_factory_function: None,
            queue_factory_glob: None,
            queue_project_factory_names: Vec::new(),
            queue_factory_roles: crate::config::v2::schema::QueueFactoryRoles::default(),
            http_prefixes: Vec::new(),
            effect_functions: HashMap::new(),
            visible_files: None,
            server_route_filter: None,
            trpc_router_glob: None,
        }
    }
}
