// The explicit macOS-dependent tests that pull requests run on macOS.
// The full sharded macOS suite runs on main. See docs/development/ci-builds.md.

const cliFixture = "fixtures/check/aggregate-agents-md-max-size";
const napiFixture = "fixtures/napi/real-addon-dependencies";

// Cargo integration-test targets or bin filters. `guards` pins test names or
// modules inside `file` so a rename fails the guard test.
const rust = [
  {
    selector: ["--test", "macos_invocation"],
    args: [],
    guards: [
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
];

// Vitest files and exact top-level test names that run against the built addon.
const api = [
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
];

module.exports = { api, cliFixture, napiFixture, rust };
