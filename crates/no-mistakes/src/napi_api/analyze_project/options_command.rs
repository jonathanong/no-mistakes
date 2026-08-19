pub(super) fn command_options(
    request: &AnalyzeReportRequest,
    options: &AnalyzeProjectOptions,
) -> AnyhowResult<String> {
    let flags = command_merge_flags(&request.report_type);
    Ok(serde_json::to_string(&merged_command_options(
        request, options, flags,
    )?)?)
}

#[derive(Clone, Copy)]
struct CommandMergeFlags {
    root: bool,
    tsconfig: bool,
    config: bool,
}

fn command_merge_flags(report_type: &str) -> CommandMergeFlags {
    match report_type {
        "importers" | "exportsOf" | "deadExports" | "callSites" | "resolveCheck" => {
            CommandMergeFlags {
                root: true,
                tsconfig: true,
                config: false,
            }
        }
        "fetches"
        | "dataPw"
        | "ciImpact"
        | "ciEnv"
        | "ciTopology"
        | "infraResourceRefs"
        | "infraOutputs"
        | "infraTestFor"
        | "swiftImporters"
        | "swiftTestTargets" => CommandMergeFlags {
            root: true,
            tsconfig: false,
            config: true,
        },
        "testsPlan" | "testsImpact" | "testsTargets" | "testsWhy" | "impactedChecks"
        | "registryExtension" | "lockfileDiff" => CommandMergeFlags {
            root: true,
            tsconfig: report_type != "lockfileDiff" && report_type != "registryExtension",
            config: report_type != "lockfileDiff",
        },
        _ => CommandMergeFlags {
            root: false,
            tsconfig: false,
            config: false,
        },
    }
}

fn merged_command_options(
    request: &AnalyzeReportRequest,
    options: &AnalyzeProjectOptions,
    flags: CommandMergeFlags,
) -> AnyhowResult<Value> {
    let mut map = request.options.clone();
    if flags.root {
        if let Some(root) = &options.root {
            map.entry("root".to_string())
                .or_insert_with(|| Value::String(root.clone()));
        }
    }
    if flags.tsconfig {
        if let Some(tsconfig) = &options.tsconfig {
            if !map.contains_key("tsconfig") {
                map.insert(
                    "tsconfig".to_string(),
                    Value::String(forwarded_tsconfig(options, tsconfig)?),
                );
            }
        }
    }
    if flags.config {
        if let Some(config) = &options.config {
            if !map.contains_key("config") {
                map.insert(
                    "config".to_string(),
                    Value::String(forwarded_config(options, config)?),
                );
            }
        }
    }
    Ok(Value::Object(map))
}
