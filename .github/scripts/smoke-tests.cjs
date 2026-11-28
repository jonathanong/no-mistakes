// The native CLI integration cases that require the platform's real runtime.
const cliFixture = "fixtures/check/aggregate-agents-md-max-size";
const cliInvocation = "crates/no-mistakes-invocation/tests/cli_invocation.rs";

const cliCases = {
  macos: [
    "hung_fsmonitor_times_out_and_releases_the_invocation_lock",
    "lockfile_git_discovery_timeout_does_not_render_partial_output",
    "millisecond_invocation_budget_stops_blocked_config_loading",
  ],
  windows: ["ordinary_check_still_finishes_inside_its_timeout"],
};

module.exports = { cliFixture, cliInvocation, cliCases };
