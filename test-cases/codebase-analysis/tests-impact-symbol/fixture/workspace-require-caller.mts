const { parseDate } = require("@repo/dates");

export function renderWorkspaceRequiredDate(input: string) {
  return parseDate(input).toISOString();
}
