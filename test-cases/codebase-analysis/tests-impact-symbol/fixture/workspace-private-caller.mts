import { parseDate } from "@repo/dates";

export const workspaceDate = "2026-01-01";

function renderWorkspaceDate(input: string) {
  return parseDate(input).toISOString();
}

export const renderedWorkspaceDate = renderWorkspaceDate(workspaceDate);
