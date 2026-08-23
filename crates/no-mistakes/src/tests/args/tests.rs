use super::{ImpactFormat, PlanFormat};

#[test]
fn impact_formats_map_to_the_shared_renderer_without_explain() {
    for (impact, plan) in [
        (ImpactFormat::Json, PlanFormat::Json),
        (ImpactFormat::Paths, PlanFormat::Paths),
        (ImpactFormat::Commands, PlanFormat::Commands),
        (ImpactFormat::Markdown, PlanFormat::Markdown),
        (ImpactFormat::Md, PlanFormat::Md),
    ] {
        assert_eq!(PlanFormat::from(impact), plan);
    }
}
