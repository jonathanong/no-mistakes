/// Parse only static, project-mode `tsc --noEmit` arguments.
///
/// A default tsconfig is inferred only when every non-option token is known to
/// be an option value. Source-file inputs, config bypasses, non-typechecking
/// modes, and unknown options with a possible value are rejected instead of
/// being guessed as a CI gate.
pub(super) fn project_argument(arguments: &[String]) -> Option<String> {
    let mut project = None;
    let mut no_emit = false;
    let mut index = 0;
    while let Some(argument) = arguments.get(index) {
        if is_non_typechecking_mode(argument) {
            return None;
        }
        if argument == "--noEmit" {
            no_emit = true;
        } else if let Some(value) = argument.strip_prefix("--project=") {
            if value.is_empty() || project.replace(value.to_string()).is_some() {
                return None;
            }
        } else if argument == "-p" {
            return None;
        } else if argument == "--project" {
            let value = arguments.get(index + 1)?;
            if value.starts_with('-') || project.replace(value.clone()).is_some() {
                return None;
            }
            index += 1;
        } else if let Some(raw_option) = argument.strip_prefix("--") {
            let (option, inline_value) = raw_option
                .split_once('=')
                .map_or((raw_option, None), |(option, value)| (option, Some(value)));
            let value_kind = option_value_kind(option)?;
            match (value_kind, inline_value) {
                (_, Some("")) => return None,
                (OptionValueKind::Required, Some(_)) => {}
                (OptionValueKind::Required, None) => {
                    let value = arguments.get(index + 1)?;
                    if value.starts_with('-') {
                        return None;
                    }
                    index += 1;
                }
                (OptionValueKind::OptionalBoolean, Some("true" | "false")) => {}
                (OptionValueKind::OptionalBoolean, Some(_)) => return None,
                (OptionValueKind::OptionalBoolean, None)
                    if arguments
                        .get(index + 1)
                        .is_some_and(|value| matches!(value.as_str(), "true" | "false")) =>
                {
                    index += 1;
                }
                (OptionValueKind::OptionalBoolean, None) => {}
            }
        } else {
            return None;
        }
        index += 1;
    }
    no_emit.then(|| project.unwrap_or_else(|| "tsconfig.json".to_string()))
}

fn is_non_typechecking_mode(argument: &str) -> bool {
    let option = argument
        .split_once('=')
        .map_or(argument, |(option, _)| option);
    matches!(
        option,
        "--noCheck"
            | "--listFilesOnly"
            | "--ignoreConfig"
            | "--showConfig"
            | "--help"
            | "-h"
            | "--version"
            | "-v"
            | "--init"
    )
}

/// Static `tsc` options that consume one following token. This deliberately
/// excludes project-mode-breaking flags and leaves unknown value-taking forms
/// unresolved rather than mistaking a source input for configuration.
enum OptionValueKind {
    Required,
    OptionalBoolean,
}

fn option_value_kind(option: &str) -> Option<OptionValueKind> {
    matches!(
        option,
        "baseUrl"
            | "charset"
            | "declarationDir"
            | "generateCpuProfile"
            | "generateTrace"
            | "importsNotUsedAsValues"
            | "jsx"
            | "jsxFactory"
            | "jsxFragmentFactory"
            | "jsxImportSource"
            | "lib"
            | "locale"
            | "mapRoot"
            | "maxNodeModuleJsDepth"
            | "module"
            | "moduleDetection"
            | "moduleResolution"
            | "newLine"
            | "outDir"
            | "outFile"
            | "paths"
            | "plugins"
            | "reactNamespace"
            | "rootDir"
            | "rootDirs"
            | "sourceRoot"
            | "target"
            | "tsBuildInfoFile"
            | "typeRoots"
            | "types"
    )
    .then_some(OptionValueKind::Required)
    .or_else(|| {
        matches!(
            option,
            "allowArbitraryExtensions"
                | "allowImportingTsExtensions"
                | "allowJs"
                | "allowSyntheticDefaultImports"
                | "allowUnreachableCode"
                | "allowUnusedLabels"
                | "alwaysStrict"
                | "checkJs"
                | "declaration"
                | "declarationMap"
                | "downlevelIteration"
                | "emitBOM"
                | "emitDeclarationOnly"
                | "erasableSyntaxOnly"
                | "esModuleInterop"
                | "exactOptionalPropertyTypes"
                | "experimentalDecorators"
                | "forceConsistentCasingInFileNames"
                | "importHelpers"
                | "inlineSourceMap"
                | "inlineSources"
                | "isolatedDeclarations"
                | "isolatedModules"
                | "listEmittedFiles"
                | "listFiles"
                | "noEmitHelpers"
                | "noEmitOnError"
                | "noErrorTruncation"
                | "noFallthroughCasesInSwitch"
                | "noImplicitAny"
                | "noImplicitOverride"
                | "noImplicitReturns"
                | "noImplicitThis"
                | "noImplicitUseStrict"
                | "noLib"
                | "noPropertyAccessFromIndexSignature"
                | "noResolve"
                | "noStrictGenericChecks"
                | "noUncheckedIndexedAccess"
                | "noUnusedLocals"
                | "noUnusedParameters"
                | "preserveConstEnums"
                | "preserveSymlinks"
                | "preserveValueImports"
                | "pretty"
                | "removeComments"
                | "resolveJsonModule"
                | "rewriteRelativeImportExtensions"
                | "skipDefaultLibCheck"
                | "skipLibCheck"
                | "sourceMap"
                | "strict"
                | "strictBindCallApply"
                | "strictBuiltinIteratorReturn"
                | "strictFunctionTypes"
                | "strictNullChecks"
                | "strictPropertyInitialization"
                | "stripInternal"
                | "traceResolution"
                | "useDefineForClassFields"
                | "useUnknownInCatchVariables"
                | "verbatimModuleSyntax"
        )
        .then_some(OptionValueKind::OptionalBoolean)
    })
}
