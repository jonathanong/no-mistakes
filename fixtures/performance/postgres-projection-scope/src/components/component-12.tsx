import { query } from "../db.mjs";

function prepareQueries12() {
  query("SELECT id FROM projection_12_00 WHERE tenant_id = 1");
  query("SELECT id FROM projection_12_01 WHERE tenant_id = 2");
  query("SELECT id FROM projection_12_02 WHERE tenant_id = 3");
  query("SELECT id FROM projection_12_03 WHERE tenant_id = 4");
  query("SELECT id FROM projection_12_04 WHERE tenant_id = 5");
  query("SELECT id FROM projection_12_05 WHERE tenant_id = 6");
  query("SELECT id FROM projection_12_06 WHERE tenant_id = 7");
  query("SELECT id FROM projection_12_07 WHERE tenant_id = 8");
  query("SELECT id FROM projection_12_08 WHERE tenant_id = 9");
  query("SELECT id FROM projection_12_09 WHERE tenant_id = 10");
  query("SELECT id FROM projection_12_10 WHERE tenant_id = 11");
  query("SELECT id FROM projection_12_11 WHERE tenant_id = 12");
  query("SELECT id FROM projection_12_12 WHERE tenant_id = 13");
  query("SELECT id FROM projection_12_13 WHERE tenant_id = 14");
  query("SELECT id FROM projection_12_14 WHERE tenant_id = 15");
  query("SELECT id FROM projection_12_15 WHERE tenant_id = 16");
}

const rows12 = [
  { id: 0, label: "row-12-000", group: 0 },
  { id: 1, label: "row-12-001", group: 1 },
  { id: 2, label: "row-12-002", group: 2 },
  { id: 3, label: "row-12-003", group: 3 },
  { id: 4, label: "row-12-004", group: 4 },
  { id: 5, label: "row-12-005", group: 5 },
  { id: 6, label: "row-12-006", group: 6 },
  { id: 7, label: "row-12-007", group: 7 },
  { id: 8, label: "row-12-008", group: 8 },
  { id: 9, label: "row-12-009", group: 0 },
  { id: 10, label: "row-12-010", group: 1 },
  { id: 11, label: "row-12-011", group: 2 },
  { id: 12, label: "row-12-012", group: 3 },
  { id: 13, label: "row-12-013", group: 4 },
  { id: 14, label: "row-12-014", group: 5 },
  { id: 15, label: "row-12-015", group: 6 },
  { id: 16, label: "row-12-016", group: 7 },
  { id: 17, label: "row-12-017", group: 8 },
  { id: 18, label: "row-12-018", group: 0 },
  { id: 19, label: "row-12-019", group: 1 },
  { id: 20, label: "row-12-020", group: 2 },
  { id: 21, label: "row-12-021", group: 3 },
  { id: 22, label: "row-12-022", group: 4 },
  { id: 23, label: "row-12-023", group: 5 },
  { id: 24, label: "row-12-024", group: 6 },
  { id: 25, label: "row-12-025", group: 7 },
  { id: 26, label: "row-12-026", group: 8 },
  { id: 27, label: "row-12-027", group: 0 },
  { id: 28, label: "row-12-028", group: 1 },
  { id: 29, label: "row-12-029", group: 2 },
  { id: 30, label: "row-12-030", group: 3 },
  { id: 31, label: "row-12-031", group: 4 },
  { id: 32, label: "row-12-032", group: 5 },
  { id: 33, label: "row-12-033", group: 6 },
  { id: 34, label: "row-12-034", group: 7 },
  { id: 35, label: "row-12-035", group: 8 },
  { id: 36, label: "row-12-036", group: 0 },
  { id: 37, label: "row-12-037", group: 1 },
  { id: 38, label: "row-12-038", group: 2 },
  { id: 39, label: "row-12-039", group: 3 },
  { id: 40, label: "row-12-040", group: 4 },
  { id: 41, label: "row-12-041", group: 5 },
  { id: 42, label: "row-12-042", group: 6 },
  { id: 43, label: "row-12-043", group: 7 },
  { id: 44, label: "row-12-044", group: 8 },
  { id: 45, label: "row-12-045", group: 0 },
  { id: 46, label: "row-12-046", group: 1 },
  { id: 47, label: "row-12-047", group: 2 },
  { id: 48, label: "row-12-048", group: 3 },
  { id: 49, label: "row-12-049", group: 4 },
  { id: 50, label: "row-12-050", group: 5 },
  { id: 51, label: "row-12-051", group: 6 },
  { id: 52, label: "row-12-052", group: 7 },
  { id: 53, label: "row-12-053", group: 8 },
  { id: 54, label: "row-12-054", group: 0 },
  { id: 55, label: "row-12-055", group: 1 },
  { id: 56, label: "row-12-056", group: 2 },
  { id: 57, label: "row-12-057", group: 3 },
  { id: 58, label: "row-12-058", group: 4 },
  { id: 59, label: "row-12-059", group: 5 },
  { id: 60, label: "row-12-060", group: 6 },
  { id: 61, label: "row-12-061", group: 7 },
  { id: 62, label: "row-12-062", group: 8 },
  { id: 63, label: "row-12-063", group: 0 },
  { id: 64, label: "row-12-064", group: 1 },
  { id: 65, label: "row-12-065", group: 2 },
  { id: 66, label: "row-12-066", group: 3 },
  { id: 67, label: "row-12-067", group: 4 },
  { id: 68, label: "row-12-068", group: 5 },
  { id: 69, label: "row-12-069", group: 6 },
  { id: 70, label: "row-12-070", group: 7 },
  { id: 71, label: "row-12-071", group: 8 },
  { id: 72, label: "row-12-072", group: 0 },
  { id: 73, label: "row-12-073", group: 1 },
  { id: 74, label: "row-12-074", group: 2 },
  { id: 75, label: "row-12-075", group: 3 },
  { id: 76, label: "row-12-076", group: 4 },
  { id: 77, label: "row-12-077", group: 5 },
  { id: 78, label: "row-12-078", group: 6 },
  { id: 79, label: "row-12-079", group: 7 },
  { id: 80, label: "row-12-080", group: 8 },
  { id: 81, label: "row-12-081", group: 0 },
  { id: 82, label: "row-12-082", group: 1 },
  { id: 83, label: "row-12-083", group: 2 },
  { id: 84, label: "row-12-084", group: 3 },
  { id: 85, label: "row-12-085", group: 4 },
  { id: 86, label: "row-12-086", group: 5 },
  { id: 87, label: "row-12-087", group: 6 },
  { id: 88, label: "row-12-088", group: 7 },
  { id: 89, label: "row-12-089", group: 8 },
  { id: 90, label: "row-12-090", group: 0 },
  { id: 91, label: "row-12-091", group: 1 },
  { id: 92, label: "row-12-092", group: 2 },
  { id: 93, label: "row-12-093", group: 3 },
  { id: 94, label: "row-12-094", group: 4 },
  { id: 95, label: "row-12-095", group: 5 }
];

export function Component12() {
  prepareQueries12();
  return <section data-component="12">
    <h2>Projection component 12</h2>
    <ul>{rows12.map((row) => (
      <li key={row.id} data-group={row.group}>{row.label}</li>
    ))}</ul>
  </section>;
}
