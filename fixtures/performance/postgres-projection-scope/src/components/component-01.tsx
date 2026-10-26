import { query } from "../db.mjs";

function prepareQueries01() {
  query("SELECT id FROM projection_01_00 WHERE tenant_id = 1");
  query("SELECT id FROM projection_01_01 WHERE tenant_id = 2");
  query("SELECT id FROM projection_01_02 WHERE tenant_id = 3");
  query("SELECT id FROM projection_01_03 WHERE tenant_id = 4");
  query("SELECT id FROM projection_01_04 WHERE tenant_id = 5");
  query("SELECT id FROM projection_01_05 WHERE tenant_id = 6");
  query("SELECT id FROM projection_01_06 WHERE tenant_id = 7");
  query("SELECT id FROM projection_01_07 WHERE tenant_id = 8");
  query("SELECT id FROM projection_01_08 WHERE tenant_id = 9");
  query("SELECT id FROM projection_01_09 WHERE tenant_id = 10");
  query("SELECT id FROM projection_01_10 WHERE tenant_id = 11");
  query("SELECT id FROM projection_01_11 WHERE tenant_id = 12");
  query("SELECT id FROM projection_01_12 WHERE tenant_id = 13");
  query("SELECT id FROM projection_01_13 WHERE tenant_id = 14");
  query("SELECT id FROM projection_01_14 WHERE tenant_id = 15");
  query("SELECT id FROM projection_01_15 WHERE tenant_id = 16");
}

const rows01 = [
  { id: 0, label: "row-01-000", group: 0 },
  { id: 1, label: "row-01-001", group: 1 },
  { id: 2, label: "row-01-002", group: 2 },
  { id: 3, label: "row-01-003", group: 3 },
  { id: 4, label: "row-01-004", group: 4 },
  { id: 5, label: "row-01-005", group: 5 },
  { id: 6, label: "row-01-006", group: 6 },
  { id: 7, label: "row-01-007", group: 7 },
  { id: 8, label: "row-01-008", group: 8 },
  { id: 9, label: "row-01-009", group: 0 },
  { id: 10, label: "row-01-010", group: 1 },
  { id: 11, label: "row-01-011", group: 2 },
  { id: 12, label: "row-01-012", group: 3 },
  { id: 13, label: "row-01-013", group: 4 },
  { id: 14, label: "row-01-014", group: 5 },
  { id: 15, label: "row-01-015", group: 6 },
  { id: 16, label: "row-01-016", group: 7 },
  { id: 17, label: "row-01-017", group: 8 },
  { id: 18, label: "row-01-018", group: 0 },
  { id: 19, label: "row-01-019", group: 1 },
  { id: 20, label: "row-01-020", group: 2 },
  { id: 21, label: "row-01-021", group: 3 },
  { id: 22, label: "row-01-022", group: 4 },
  { id: 23, label: "row-01-023", group: 5 },
  { id: 24, label: "row-01-024", group: 6 },
  { id: 25, label: "row-01-025", group: 7 },
  { id: 26, label: "row-01-026", group: 8 },
  { id: 27, label: "row-01-027", group: 0 },
  { id: 28, label: "row-01-028", group: 1 },
  { id: 29, label: "row-01-029", group: 2 },
  { id: 30, label: "row-01-030", group: 3 },
  { id: 31, label: "row-01-031", group: 4 },
  { id: 32, label: "row-01-032", group: 5 },
  { id: 33, label: "row-01-033", group: 6 },
  { id: 34, label: "row-01-034", group: 7 },
  { id: 35, label: "row-01-035", group: 8 },
  { id: 36, label: "row-01-036", group: 0 },
  { id: 37, label: "row-01-037", group: 1 },
  { id: 38, label: "row-01-038", group: 2 },
  { id: 39, label: "row-01-039", group: 3 },
  { id: 40, label: "row-01-040", group: 4 },
  { id: 41, label: "row-01-041", group: 5 },
  { id: 42, label: "row-01-042", group: 6 },
  { id: 43, label: "row-01-043", group: 7 },
  { id: 44, label: "row-01-044", group: 8 },
  { id: 45, label: "row-01-045", group: 0 },
  { id: 46, label: "row-01-046", group: 1 },
  { id: 47, label: "row-01-047", group: 2 },
  { id: 48, label: "row-01-048", group: 3 },
  { id: 49, label: "row-01-049", group: 4 },
  { id: 50, label: "row-01-050", group: 5 },
  { id: 51, label: "row-01-051", group: 6 },
  { id: 52, label: "row-01-052", group: 7 },
  { id: 53, label: "row-01-053", group: 8 },
  { id: 54, label: "row-01-054", group: 0 },
  { id: 55, label: "row-01-055", group: 1 },
  { id: 56, label: "row-01-056", group: 2 },
  { id: 57, label: "row-01-057", group: 3 },
  { id: 58, label: "row-01-058", group: 4 },
  { id: 59, label: "row-01-059", group: 5 },
  { id: 60, label: "row-01-060", group: 6 },
  { id: 61, label: "row-01-061", group: 7 },
  { id: 62, label: "row-01-062", group: 8 },
  { id: 63, label: "row-01-063", group: 0 },
  { id: 64, label: "row-01-064", group: 1 },
  { id: 65, label: "row-01-065", group: 2 },
  { id: 66, label: "row-01-066", group: 3 },
  { id: 67, label: "row-01-067", group: 4 },
  { id: 68, label: "row-01-068", group: 5 },
  { id: 69, label: "row-01-069", group: 6 },
  { id: 70, label: "row-01-070", group: 7 },
  { id: 71, label: "row-01-071", group: 8 },
  { id: 72, label: "row-01-072", group: 0 },
  { id: 73, label: "row-01-073", group: 1 },
  { id: 74, label: "row-01-074", group: 2 },
  { id: 75, label: "row-01-075", group: 3 },
  { id: 76, label: "row-01-076", group: 4 },
  { id: 77, label: "row-01-077", group: 5 },
  { id: 78, label: "row-01-078", group: 6 },
  { id: 79, label: "row-01-079", group: 7 },
  { id: 80, label: "row-01-080", group: 8 },
  { id: 81, label: "row-01-081", group: 0 },
  { id: 82, label: "row-01-082", group: 1 },
  { id: 83, label: "row-01-083", group: 2 },
  { id: 84, label: "row-01-084", group: 3 },
  { id: 85, label: "row-01-085", group: 4 },
  { id: 86, label: "row-01-086", group: 5 },
  { id: 87, label: "row-01-087", group: 6 },
  { id: 88, label: "row-01-088", group: 7 },
  { id: 89, label: "row-01-089", group: 8 },
  { id: 90, label: "row-01-090", group: 0 },
  { id: 91, label: "row-01-091", group: 1 },
  { id: 92, label: "row-01-092", group: 2 },
  { id: 93, label: "row-01-093", group: 3 },
  { id: 94, label: "row-01-094", group: 4 },
  { id: 95, label: "row-01-095", group: 5 }
];

export function Component01() {
  prepareQueries01();
  return <section data-component="01">
    <h2>Projection component 01</h2>
    <ul>{rows01.map((row) => (
      <li key={row.id} data-group={row.group}>{row.label}</li>
    ))}</ul>
  </section>;
}
