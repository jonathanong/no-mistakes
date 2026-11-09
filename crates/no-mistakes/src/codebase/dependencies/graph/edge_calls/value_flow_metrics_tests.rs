pub(crate) fn builtin_callback_sites() -> usize {
    super::LAST_BUILTIN_CALLBACK_SITES.get()
}

pub(crate) fn callback_ownership_cache_initialized() -> bool {
    super::LAST_CALLBACK_OWNERSHIP_CACHE.get()
}
