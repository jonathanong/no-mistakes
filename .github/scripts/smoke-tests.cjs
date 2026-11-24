// The explicit OS-dependent tests that pull requests run on each native OS.
// The full sharded suite runs on main. See docs/development/ci-builds.md.

const cliFixture = "fixtures/check/aggregate-agents-md-max-size";
const napiFixture = "fixtures/napi/real-addon-dependencies";

// Per OS: `label` is the RUNNER_OS value (cargo profile selection). `rust` lists
// Cargo integration-test targets or bin filters; `guards` pins test names or
// modules inside `file` so a rename fails the guard test. `api` lists Vitest
// files and exact top-level test names that run against the built addon.
const macos = {
  label: "macOS",
  rust: [
    {
      selector: ["--test", "macos_invocation"],
      args: [],
      guards: [
        {
          file: "crates/no-mistakes/Cargo.toml",
          text: 'name = "macos_invocation"\npath = "tests/macos_invocation.rs"',
        },
        {
          file: "crates/no-mistakes/tests/macos_invocation.rs",
          text: '#[path = "cli_invocation.rs"]\nmod cli_invocation;',
        },
        {
          file: "crates/no-mistakes/tests/cli_invocation.rs",
          text: "fn hung_fsmonitor_times_out_and_releases_the_invocation_lock",
        },
        {
          file: "crates/no-mistakes/tests/cli_invocation.rs",
          text: "fn lockfile_git_discovery_timeout_does_not_render_partial_output",
        },
      ],
    },
    {
      selector: ["--bin", "no-mistakes"],
      args: ["invocation::tests::blocked_check::"],
      guards: [
        { file: "crates/no-mistakes/src/main/invocation/tests.rs", text: "mod blocked_check;" },
      ],
    },
  ],
  api: [
    {
      file: "packages/no-mistakes/scripts/real-napi-api.test.js",
      names: [
        "compiled internal N-API lock serializes separate Node processes",
        "compiled internal N-API lock is released when a worker terminates",
        "compiled internal N-API rename preserves an existing destination and claims a vacant one",
      ],
    },
    {
      file: "packages/no-mistakes/scripts/invocation-diagnostics-api.test.js",
      names: [
        "compiled annotation collectDiagnostics preserves default reports and async CLI parity",
      ],
    },
  ],
};

const jobObject = "crates/no-mistakes/tests/windows_job_object.rs";
const consoleFile = "crates/no-mistakes/tests/windows_job_object/deadline_console.rs";
const invocation = "crates/no-mistakes/tests/cli_invocation.rs";
const nativePackage = "packages/no-mistakes/scripts/native-package.test.js";

const windows = {
  label: "Windows",
  rust: [
    {
      selector: ["--test", "windows_job_object"],
      args: ["--", "--test-threads=1"],
      guards: [
        { file: jobObject, text: "fn command_output_resumes_child_after_job_assignment" },
        { file: consoleFile, text: "fn redirected_deadline_output_remains_utf8" },
        { file: consoleFile, text: "fn native_console_ignores_the_oem_output_code_page" },
      ],
    },
    {
      selector: ["--test", "windows_invocation"],
      args: [],
      guards: [
        { file: "crates/no-mistakes/tests/windows_invocation.rs", text: "mod cli_invocation;" },
        {
          file: invocation,
          text: "fn invocation_help_documents_independent_timeouts_and_lock_failure",
        },
        { file: invocation, text: "fn ordinary_check_still_finishes_inside_its_timeout" },
      ],
    },
  ],
  api: [
    {
      file: "packages/no-mistakes/scripts/real-napi-api.test.js",
      names: [
        "compiled internal N-API lock serializes separate Node processes",
        "compiled internal N-API lock is released when a worker terminates",
        "compiled internal N-API rename preserves an existing destination and claims a vacant one",
      ],
    },
    {
      file: "packages/no-mistakes/scripts/check-policy-test-helpers.test.js",
      names: ["healthy reports retain identity and drive and UNC paths retain their meaning"],
    },
    {
      file: nativePackage,
      names: [
        "resolves the Windows executable from the same optional package",
        "Windows verifies CLI presence without Unix execute-bit semantics",
      ],
    },
  ],
};

module.exports = { cliFixture, napiFixture, platforms: { macos, windows } };
