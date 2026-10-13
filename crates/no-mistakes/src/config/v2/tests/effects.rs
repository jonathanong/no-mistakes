use super::*;

#[test]
fn effect_module_export_targets_round_trip_and_default_empty() {
    let config: NoMistakesConfig = serde_yaml::from_str("effects: {storage: {targets: [{module: '@vendor/database', export: query, category: writes}]}}").unwrap();
    let targets = &config.effects["storage"].targets;
    assert_eq!(targets.len(), 1);
    assert_eq!(targets[0].module, "@vendor/database");
    assert_eq!(targets[0].export, "query");
    assert_eq!(targets[0].category.as_deref(), Some("writes"));
    assert_eq!(
        config,
        serde_yaml::from_str(&serde_yaml::to_string(&config).unwrap()).unwrap()
    );
    let legacy: NoMistakesConfig =
        serde_yaml::from_str("effects: {storage: {functions: [query]}}").unwrap();
    assert!(legacy.effects["storage"].targets.is_empty());
}

#[test]
fn per_item_effect_configuration_round_trips_without_changing_query_sinks() {
    let yaml = "effects:\n  postgres:\n    functions: [read]\n    categories: {write: [write]}\n    transactionFunctions: [tx.query]\n    batchFunctions: [readMany]\n";
    let config: NoMistakesConfig = serde_yaml::from_str(yaml).unwrap();
    let effects = &config.effects["postgres"];
    assert_eq!(effects.functions, ["read"]);
    assert_eq!(effects.categories["write"], ["write"]);
    assert_eq!(effects.transaction_functions, ["tx.query"]);
    assert_eq!(effects.batch_functions, ["readMany"]);
    let serialized = serde_yaml::to_string(&config).unwrap();
    assert_eq!(config, serde_yaml::from_str(&serialized).unwrap());
    let legacy: NoMistakesConfig =
        serde_yaml::from_str("effects: {postgres: {functions: [read]}}").unwrap();
    assert!(legacy.effects["postgres"].transaction_functions.is_empty());
    assert!(legacy.effects["postgres"].batch_functions.is_empty());
}

#[test]
fn returned_executor_selectors_round_trip_and_reject_unknown_fields() {
    let config: NoMistakesConfig = serde_yaml::from_str("effects: {postgres: {returnedExecutors: [{module: '@vendor/database', export: begin, category: transaction}]}}").unwrap();
    assert_eq!(
        config.effects["postgres"].returned_executors[0].export,
        "begin"
    );
    assert_eq!(
        config,
        serde_yaml::from_str(&serde_yaml::to_string(&config).unwrap()).unwrap()
    );
    assert!(serde_yaml::from_str::<NoMistakesConfig>(
        "effects: {postgres: {returnedExecutors: [{module: db, export: begin, unknown: true}]}}"
    )
    .is_err());
    let legacy: NoMistakesConfig =
        serde_yaml::from_str("effects: {postgres: {functions: [read]}}").unwrap();
    assert!(legacy.effects["postgres"].returned_executors.is_empty());
}
