// The explicit OS-dependent tests that pull requests run on each native OS.
// The full native suite runs on main. See docs/development/ci-builds.md.

const cliFixture = "fixtures/check/aggregate-agents-md-max-size";
const napiFixture = "fixtures/napi/real-addon-dependencies";
const invocationCrate = "crates/no-mistakes-invocation";
const cliInvocation = `${invocationCrate}/tests/cli_invocation.rs`;

function cliTests(names) {
  return {
    package: "no-mistakes-invocation",
    selector: ["--test", "cli_smoke"],
    args: ["--", "--exact", ...names.map((name) => `cli_invocation::${name}`)],
    guards: names.map((name) => ({ file: cliInvocation, text: `fn ${name}` })),
  };
}

// Per OS: `rust` lists Cargo integration-test targets and filters; `guards` pins test names or
// modules inside `file` so a rename fails the guard test. `api` lists Vitest
// files and exact top-level test names that run against the built addon.
const macos = {
  rust: [
    {
      package: "no-mistakes-invocation",
      selector: ["--test", "macos_invocation"],
      args: [],
      guards: [
        {
          file: "crates/no-mistakes-invocation/Cargo.toml",
          text: 'name = "macos_invocation"\npath = "tests/macos_invocation.rs"',
        },
        {
          file: "crates/no-mistakes-invocation/src/invocation/tests/blocked_fifo.rs",
          text: "fn watchdog_stops_blocked_fifo_read_and_names_check_phase",
        },
      ],
    },
    cliTests([
      "hung_fsmonitor_times_out_and_releases_the_invocation_lock",
      "lockfile_git_discovery_timeout_does_not_render_partial_output",
      "millisecond_invocation_budget_stops_blocked_config_loading",
    ]),
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

const jobObject = `${invocationCrate}/tests/windows_job_object.rs`;
const consoleFile = `${invocationCrate}/tests/windows_job_object/deadline_console.rs`;

const windows = {
  rust: [
    {
      package: "no-mistakes-invocation",
      selector: ["--test", "windows_job_object"],
      args: ["--", "--test-threads=1"],
      guards: [
        { file: jobObject, text: "fn command_output_resumes_child_after_job_assignment" },
        { file: consoleFile, text: "fn redirected_deadline_output_remains_utf8" },
        { file: consoleFile, text: "fn native_console_ignores_the_oem_output_code_page" },
      ],
    },
    cliTests(["ordinary_check_still_finishes_inside_its_timeout"]),
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
  ],
};

module.exports = { cliFixture, napiFixture, platforms: { macos, windows } };
