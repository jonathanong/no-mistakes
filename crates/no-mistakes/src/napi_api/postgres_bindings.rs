pub(crate) fn generate_postgres_catalog_json_impl(
    options: serde_json::Value,
) -> napi::Result<String> {
    let options =
        options::parse_options_value::<crate::postgres_catalog::PostgresCatalogOptions>(options)?;
    let catalog = crate::postgres_catalog::generate(&options).map_err(options::to_napi_error)?;
    Ok(catalog.to_string())
}
json_binding!(
    generate_postgres_catalog_json,
    "generatePostgresCatalogJson",
    generate_postgres_catalog_json_impl
);

pub(crate) fn postgres_lock_inventory_json_impl(
    options: serde_json::Value,
) -> napi::Result<String> {
    let options = options::parse_options_value::<
        crate::postgres_lock_inventory::PostgresLockInventoryOptions,
    >(options)?;
    let report = crate::postgres_lock_inventory::run(&options).map_err(options::to_napi_error)?;
    serde_json::to_string(&report).map_err(|error| options::to_napi_error(error.into()))
}
json_binding!(
    postgres_lock_inventory_json,
    "postgresLockInventoryJson",
    postgres_lock_inventory_json_impl
);

#[cfg(all(test, any(nm_shard_all, nm_shard_0)))]
#[path = "lock_inventory_tests.rs"]
mod lock_inventory_tests;
