import type {
  AnalyzeProjectOptions, CheckOptions, CheckReport, RunnerConfigDeadlineEvidence, ConfigDeadlineEvidence,
  ProjectDeadlineEvidence, DeclaredDeadlineSlot, DeclaredDeadlineUnknownReason, RunnerDeadlineStatus,
  DeadlineProvenance, DeadlineInheritanceEvidence,
} from '../../../packages/no-mistakes';
export const options: CheckOptions = { includeRunnerConfigDeadlines: true };
export function milliseconds(slot: DeclaredDeadlineSlot): number | null {
  switch (slot.status) {
    case 'known': return slot.milliseconds;
    case 'absent': return null;
    case 'unknown': throw new Error(unknownReason(slot.reason));
  }
}
export function projects(report: CheckReport): ProjectDeadlineEvidence[] {
  return (report.runnerConfigDeadlines ?? []).flatMap((runner: RunnerConfigDeadlineEvidence) =>
    runner.configs.flatMap((config: ConfigDeadlineEvidence) => config.status === 'prepared' ? config.projects : []));
}
export function inheritedPaths(provenance: DeadlineProvenance): string[] {
  return provenance.inheritedThrough.map((via: DeadlineInheritanceEvidence) => via.path);
}

export function unknownReason(reason: DeclaredDeadlineUnknownReason): string { return reason; }
export function selected(status: RunnerDeadlineStatus): boolean { return status !== 'notRequested'; }

// The public batch option must resolve its existing topology declaration transitively.
export const topologyOptions: AnalyzeProjectOptions = {
  root: ".",
  reports: [{ type: "ciTopology", id: "topology" }],
};
