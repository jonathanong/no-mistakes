// Included into `napi_api` via `include!`; shares that module's imports.
// AsyncTask wrappers for the issue-419 query commands (stripped under coverage).

json_binding!(data_pw_json, "dataPwJson", data_pw_json_impl);
json_binding!(effects_json, "effectsJson", effects_json_impl);
json_binding!(rsc_callers_json, "rscCallersJson", rsc_callers_json_impl);
json_binding!(
    registry_extension_json,
    "registryExtensionJson",
    registry_extension_json_impl
);
