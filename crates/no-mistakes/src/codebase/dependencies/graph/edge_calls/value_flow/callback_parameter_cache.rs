use super::*;

#[derive(Default)]
pub(super) struct Caches(std::sync::OnceLock<ParameterCache>);
impl Caches {
    #[cfg(feature = "test-instrumentation")]
    pub(super) fn initialized(&self) -> bool {
        self.0.get().is_some()
    }
    pub(super) fn for_site<'a>(
        &'a self,
        site: &ResolvedCallSite,
        facts: &dyn TsFactLookup,
        indexes: &CallableResolutionIndexes,
        ordinary: &'a ParameterCache,
    ) -> &'a ParameterCache {
        if site.invocation == InvocationKind::Callback
            && indexes
                .file(facts, &site.file)
                .is_some_and(|index| index.values.native_callbacks.contains(&site.offset))
        {
            self.0.get_or_init(ParameterCache::for_callback_ownership)
        } else {
            ordinary
        }
    }
    pub(super) fn invalidate(
        &self,
        ordinary: &ParameterCache,
        mut changed: FxHashSet<FunctionKey>,
    ) -> FxHashSet<FunctionKey> {
        if let Some(cache) = self.0.get() {
            loop {
                let count = changed.len();
                changed = ordinary.affected(cache.affected(changed));
                if changed.len() == count {
                    break;
                }
            }
            cache.clear();
            changed
        } else {
            changed
        }
    }
}
