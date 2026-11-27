use crate::codebase::check_facts::PlaywrightSettingsKey;
use crate::codebase::playwright_facts::selectors::AppSelector;
use crate::codebase::playwright_facts::text::AppTextTarget;
use dashmap::DashMap;
use std::sync::Arc;

pub(super) type PlaywrightScanKey = (u64, PlaywrightSettingsKey);
pub(super) type AppSelectorOccurrencesCache =
    Arc<DashMap<(PlaywrightScanKey, bool), Result<Arc<Vec<AppSelector>>, String>>>;
pub(super) type PlaywrightRoutesCache =
    Arc<DashMap<PlaywrightScanKey, Arc<Vec<crate::routes::Route>>>>;
pub(super) type AppTextTargetsCache =
    Arc<DashMap<PlaywrightScanKey, Result<Arc<Vec<AppTextTarget>>, String>>>;
pub(super) type RouteReachableFilesCache = Arc<
    DashMap<
        PlaywrightScanKey,
        Result<Arc<crate::codebase::dependencies::graph::RouteReachableFiles>, String>,
    >,
>;
