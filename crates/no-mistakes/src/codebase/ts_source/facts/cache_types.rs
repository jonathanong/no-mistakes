use crate::codebase::check_facts::PlaywrightSettingsKey;
use crate::playwright::analysis::text_types::AppTextTarget;
use crate::playwright::selectors::AppSelector;
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
