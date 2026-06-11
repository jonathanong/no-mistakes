import { formatDate } from "@repo/dates";

function parseDate(value: string) {
  return new Date(value);
}

export const formattedWorkspaceDate = formatDate(parseDate("2026-01-01"));
