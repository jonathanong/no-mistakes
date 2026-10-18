#[cfg(feature = "test-instrumentation")]
pub use value_flow_metrics::last_evaluations as benchmark_probe_value_flow_evaluations;

#[cfg(feature = "test-instrumentation")]
pub use value_flow_metrics::parameter_agreements as benchmark_probe_parameter_agreements;

#[cfg(feature = "test-instrumentation")]
pub use value_flow_metrics::origin_nodes as benchmark_probe_origin_nodes;

#[cfg(feature = "test-instrumentation")]
pub use value_flow_metrics::dependency_links as benchmark_probe_parameter_dependency_links;

#[cfg(feature = "test-instrumentation")]
pub use value_flow_metrics::{
    replacement_snapshot as benchmark_probe_replacements, ReplacementIncomingProbe,
    ReplacementPhysicalProbe, ReplacementProbeSnapshot, ReplacementRhsProbe,
    ReplacementSourceCallProbe,
};

#[cfg(feature = "test-instrumentation")]
pub use value_flow_metrics::execution_steps as benchmark_probe_execution_phase_steps;

#[cfg(feature = "test-instrumentation")]
pub use value_flow::admission_mode::with_original_admission as benchmark_probe_with_original_admission;

#[cfg(all(test, feature = "test-instrumentation"))]
pub use value_flow::opaque_entry_mode::with_original_entries as benchmark_probe_with_original_entries;

#[cfg(feature = "test-instrumentation")]
pub use value_flow_metrics::module_provider_checks as benchmark_probe_module_provider_checks;
