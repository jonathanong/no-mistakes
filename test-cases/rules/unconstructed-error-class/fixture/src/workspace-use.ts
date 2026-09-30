import { LibBaseError, WorkspaceError } from "@fixture/lib";

export const fromWorkspace = new WorkspaceError("workspace");

// Flagged: a subclass of a workspace package class that nothing constructs.
export class LocalFromLib extends LibBaseError {}
