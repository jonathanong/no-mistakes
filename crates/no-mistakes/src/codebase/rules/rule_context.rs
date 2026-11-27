use crate::codebase::check_facts::CheckFactMap;
use crate::codebase::ts_source::SourceStore;
use crate::config::v2::NoMistakesConfig;
use no_mistakes_rule_kernel::rule_context as kernel;

pub(crate) type SourcedRuleContext<'a> =
    kernel::SourcedRuleContext<'a, NoMistakesConfig, SourceStore>;
pub(crate) type PreparedRuleContext<'a> =
    kernel::PreparedRuleContext<'a, NoMistakesConfig, SourceStore, CheckFactMap>;
