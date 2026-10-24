import { query } from "../db.mjs";

function prepareQueries16() {
  query("SELECT id FROM projection_16_00 WHERE tenant_id = 1");
  query("SELECT id FROM projection_16_01 WHERE tenant_id = 2");
  query("SELECT id FROM projection_16_02 WHERE tenant_id = 3");
  query("SELECT id FROM projection_16_03 WHERE tenant_id = 4");
  query("SELECT id FROM projection_16_04 WHERE tenant_id = 5");
  query("SELECT id FROM projection_16_05 WHERE tenant_id = 6");
  query("SELECT id FROM projection_16_06 WHERE tenant_id = 7");
  query("SELECT id FROM projection_16_07 WHERE tenant_id = 8");
  query("SELECT id FROM projection_16_08 WHERE tenant_id = 9");
  query("SELECT id FROM projection_16_09 WHERE tenant_id = 10");
  query("SELECT id FROM projection_16_10 WHERE tenant_id = 11");
  query("SELECT id FROM projection_16_11 WHERE tenant_id = 12");
  query("SELECT id FROM projection_16_12 WHERE tenant_id = 13");
  query("SELECT id FROM projection_16_13 WHERE tenant_id = 14");
  query("SELECT id FROM projection_16_14 WHERE tenant_id = 15");
  query("SELECT id FROM projection_16_15 WHERE tenant_id = 16");
}

const rows16 = [
  { id: 0, label: "row-16-000", group: 0 },
  { id: 1, label: "row-16-001", group: 1 },
  { id: 2, label: "row-16-002", group: 2 },
  { id: 3, label: "row-16-003", group: 3 },
  { id: 4, label: "row-16-004", group: 4 },
  { id: 5, label: "row-16-005", group: 5 },
  { id: 6, label: "row-16-006", group: 6 },
  { id: 7, label: "row-16-007", group: 7 },
  { id: 8, label: "row-16-008", group: 8 },
  { id: 9, label: "row-16-009", group: 0 },
  { id: 10, label: "row-16-010", group: 1 },
  { id: 11, label: "row-16-011", group: 2 },
  { id: 12, label: "row-16-012", group: 3 },
  { id: 13, label: "row-16-013", group: 4 },
  { id: 14, label: "row-16-014", group: 5 },
  { id: 15, label: "row-16-015", group: 6 },
  { id: 16, label: "row-16-016", group: 7 },
  { id: 17, label: "row-16-017", group: 8 },
  { id: 18, label: "row-16-018", group: 0 },
  { id: 19, label: "row-16-019", group: 1 },
  { id: 20, label: "row-16-020", group: 2 },
  { id: 21, label: "row-16-021", group: 3 },
  { id: 22, label: "row-16-022", group: 4 },
  { id: 23, label: "row-16-023", group: 5 },
  { id: 24, label: "row-16-024", group: 6 },
  { id: 25, label: "row-16-025", group: 7 },
  { id: 26, label: "row-16-026", group: 8 },
  { id: 27, label: "row-16-027", group: 0 },
  { id: 28, label: "row-16-028", group: 1 },
  { id: 29, label: "row-16-029", group: 2 },
  { id: 30, label: "row-16-030", group: 3 },
  { id: 31, label: "row-16-031", group: 4 },
  { id: 32, label: "row-16-032", group: 5 },
  { id: 33, label: "row-16-033", group: 6 },
  { id: 34, label: "row-16-034", group: 7 },
  { id: 35, label: "row-16-035", group: 8 },
  { id: 36, label: "row-16-036", group: 0 },
  { id: 37, label: "row-16-037", group: 1 },
  { id: 38, label: "row-16-038", group: 2 },
  { id: 39, label: "row-16-039", group: 3 },
  { id: 40, label: "row-16-040", group: 4 },
  { id: 41, label: "row-16-041", group: 5 },
  { id: 42, label: "row-16-042", group: 6 },
  { id: 43, label: "row-16-043", group: 7 },
  { id: 44, label: "row-16-044", group: 8 },
  { id: 45, label: "row-16-045", group: 0 },
  { id: 46, label: "row-16-046", group: 1 },
  { id: 47, label: "row-16-047", group: 2 },
  { id: 48, label: "row-16-048", group: 3 },
  { id: 49, label: "row-16-049", group: 4 },
  { id: 50, label: "row-16-050", group: 5 },
  { id: 51, label: "row-16-051", group: 6 },
  { id: 52, label: "row-16-052", group: 7 },
  { id: 53, label: "row-16-053", group: 8 },
  { id: 54, label: "row-16-054", group: 0 },
  { id: 55, label: "row-16-055", group: 1 },
  { id: 56, label: "row-16-056", group: 2 },
  { id: 57, label: "row-16-057", group: 3 },
  { id: 58, label: "row-16-058", group: 4 },
  { id: 59, label: "row-16-059", group: 5 },
  { id: 60, label: "row-16-060", group: 6 },
  { id: 61, label: "row-16-061", group: 7 },
  { id: 62, label: "row-16-062", group: 8 },
  { id: 63, label: "row-16-063", group: 0 },
  { id: 64, label: "row-16-064", group: 1 },
  { id: 65, label: "row-16-065", group: 2 },
  { id: 66, label: "row-16-066", group: 3 },
  { id: 67, label: "row-16-067", group: 4 },
  { id: 68, label: "row-16-068", group: 5 },
  { id: 69, label: "row-16-069", group: 6 },
  { id: 70, label: "row-16-070", group: 7 },
  { id: 71, label: "row-16-071", group: 8 },
  { id: 72, label: "row-16-072", group: 0 },
  { id: 73, label: "row-16-073", group: 1 },
  { id: 74, label: "row-16-074", group: 2 },
  { id: 75, label: "row-16-075", group: 3 },
  { id: 76, label: "row-16-076", group: 4 },
  { id: 77, label: "row-16-077", group: 5 },
  { id: 78, label: "row-16-078", group: 6 },
  { id: 79, label: "row-16-079", group: 7 },
  { id: 80, label: "row-16-080", group: 8 },
  { id: 81, label: "row-16-081", group: 0 },
  { id: 82, label: "row-16-082", group: 1 },
  { id: 83, label: "row-16-083", group: 2 },
  { id: 84, label: "row-16-084", group: 3 },
  { id: 85, label: "row-16-085", group: 4 },
  { id: 86, label: "row-16-086", group: 5 },
  { id: 87, label: "row-16-087", group: 6 },
  { id: 88, label: "row-16-088", group: 7 },
  { id: 89, label: "row-16-089", group: 8 },
  { id: 90, label: "row-16-090", group: 0 },
  { id: 91, label: "row-16-091", group: 1 },
  { id: 92, label: "row-16-092", group: 2 },
  { id: 93, label: "row-16-093", group: 3 },
  { id: 94, label: "row-16-094", group: 4 },
  { id: 95, label: "row-16-095", group: 5 }
];

export function Component16() {
  prepareQueries16();
  return <section data-component="16">
    <h2>Projection component 16</h2>
    <ul>{rows16.map((row) => (
      <li key={row.id} data-group={row.group}>{row.label}</li>
    ))}</ul>
  </section>;
}
