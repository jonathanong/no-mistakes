// Artifact-dataflow types for `ciTopology()`, split out of
// `workflow-topology-types.d.ts` to stay under the 200-line file limit.
// Mirrors the schema-v1 JSON contract exactly (field names, casing) — see
// `docs/node-api.md` for the stability guarantees.

export type ArtifactValue =
  | { kind: "static"; raw: string; value: string; instanceCount?: number }
  | { kind: "finite"; raw: string; values: string[]; instanceCounts: Record<string, number> }
  | { kind: "dynamic"; raw: string }
  | { kind: "path-derived"; reason: "archive-disabled" };

export type ArtifactActionFlag =
  | { kind: "static"; raw?: string; effective: boolean }
  | { kind: "dynamic"; raw: string };

export interface ArtifactUploadDeclaration {
  kind: "upload";
  name: ArtifactValue;
  archive: ArtifactActionFlag;
  overwrite: ArtifactActionFlag;
}

export type ArtifactDownloadSelector =
  | { kind: "name"; name: ArtifactValue }
  | { kind: "pattern"; pattern: ArtifactValue }
  | { kind: "all" }
  | { kind: "artifact-ids"; artifactIds: ArtifactValue }
  | {
      kind: "unresolved";
      reason: "name-with-artifact-ids";
      name: ArtifactValue;
      artifactIds: ArtifactValue;
    };

export type ArtifactDownloadSource =
  | { kind: "current-run"; repository?: ArtifactValue; runId?: ArtifactValue }
  | { kind: "external"; repository?: ArtifactValue; runId?: ArtifactValue }
  | { kind: "dynamic"; repository?: ArtifactValue; runId?: ArtifactValue };

export interface ArtifactDownloadDeclaration {
  kind: "download";
  selector: ArtifactDownloadSelector;
  source: ArtifactDownloadSource;
}

export type ArtifactDeclaration = ArtifactUploadDeclaration | ArtifactDownloadDeclaration;

export interface ArtifactEdge {
  kind: "artifact";
  from: string;
  to: string;
  name: string;
  producerStep: number;
  consumerStep: number;
  match: "exact" | "pattern" | "all" | "possible";
}
