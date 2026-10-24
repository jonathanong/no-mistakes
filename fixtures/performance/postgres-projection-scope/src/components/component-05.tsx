import { query } from "../db.mjs";

function prepareQueries05() {
  query("SELECT id FROM projection_05_00 WHERE tenant_id = 1");
  query("SELECT id FROM projection_05_01 WHERE tenant_id = 2");
  query("SELECT id FROM projection_05_02 WHERE tenant_id = 3");
  query("SELECT id FROM projection_05_03 WHERE tenant_id = 4");
  query("SELECT id FROM projection_05_04 WHERE tenant_id = 5");
  query("SELECT id FROM projection_05_05 WHERE tenant_id = 6");
  query("SELECT id FROM projection_05_06 WHERE tenant_id = 7");
  query("SELECT id FROM projection_05_07 WHERE tenant_id = 8");
  query("SELECT id FROM projection_05_08 WHERE tenant_id = 9");
  query("SELECT id FROM projection_05_09 WHERE tenant_id = 10");
  query("SELECT id FROM projection_05_10 WHERE tenant_id = 11");
  query("SELECT id FROM projection_05_11 WHERE tenant_id = 12");
  query("SELECT id FROM projection_05_12 WHERE tenant_id = 13");
  query("SELECT id FROM projection_05_13 WHERE tenant_id = 14");
  query("SELECT id FROM projection_05_14 WHERE tenant_id = 15");
  query("SELECT id FROM projection_05_15 WHERE tenant_id = 16");
}

const rows05 = [
  { id: 0, label: "row-05-000", group: 0 },
  { id: 1, label: "row-05-001", group: 1 },
  { id: 2, label: "row-05-002", group: 2 },
  { id: 3, label: "row-05-003", group: 3 },
  { id: 4, label: "row-05-004", group: 4 },
  { id: 5, label: "row-05-005", group: 5 },
  { id: 6, label: "row-05-006", group: 6 },
  { id: 7, label: "row-05-007", group: 7 },
  { id: 8, label: "row-05-008", group: 8 },
  { id: 9, label: "row-05-009", group: 0 },
  { id: 10, label: "row-05-010", group: 1 },
  { id: 11, label: "row-05-011", group: 2 },
  { id: 12, label: "row-05-012", group: 3 },
  { id: 13, label: "row-05-013", group: 4 },
  { id: 14, label: "row-05-014", group: 5 },
  { id: 15, label: "row-05-015", group: 6 },
  { id: 16, label: "row-05-016", group: 7 },
  { id: 17, label: "row-05-017", group: 8 },
  { id: 18, label: "row-05-018", group: 0 },
  { id: 19, label: "row-05-019", group: 1 },
  { id: 20, label: "row-05-020", group: 2 },
  { id: 21, label: "row-05-021", group: 3 },
  { id: 22, label: "row-05-022", group: 4 },
  { id: 23, label: "row-05-023", group: 5 },
  { id: 24, label: "row-05-024", group: 6 },
  { id: 25, label: "row-05-025", group: 7 },
  { id: 26, label: "row-05-026", group: 8 },
  { id: 27, label: "row-05-027", group: 0 },
  { id: 28, label: "row-05-028", group: 1 },
  { id: 29, label: "row-05-029", group: 2 },
  { id: 30, label: "row-05-030", group: 3 },
  { id: 31, label: "row-05-031", group: 4 },
  { id: 32, label: "row-05-032", group: 5 },
  { id: 33, label: "row-05-033", group: 6 },
  { id: 34, label: "row-05-034", group: 7 },
  { id: 35, label: "row-05-035", group: 8 },
  { id: 36, label: "row-05-036", group: 0 },
  { id: 37, label: "row-05-037", group: 1 },
  { id: 38, label: "row-05-038", group: 2 },
  { id: 39, label: "row-05-039", group: 3 },
  { id: 40, label: "row-05-040", group: 4 },
  { id: 41, label: "row-05-041", group: 5 },
  { id: 42, label: "row-05-042", group: 6 },
  { id: 43, label: "row-05-043", group: 7 },
  { id: 44, label: "row-05-044", group: 8 },
  { id: 45, label: "row-05-045", group: 0 },
  { id: 46, label: "row-05-046", group: 1 },
  { id: 47, label: "row-05-047", group: 2 },
  { id: 48, label: "row-05-048", group: 3 },
  { id: 49, label: "row-05-049", group: 4 },
  { id: 50, label: "row-05-050", group: 5 },
  { id: 51, label: "row-05-051", group: 6 },
  { id: 52, label: "row-05-052", group: 7 },
  { id: 53, label: "row-05-053", group: 8 },
  { id: 54, label: "row-05-054", group: 0 },
  { id: 55, label: "row-05-055", group: 1 },
  { id: 56, label: "row-05-056", group: 2 },
  { id: 57, label: "row-05-057", group: 3 },
  { id: 58, label: "row-05-058", group: 4 },
  { id: 59, label: "row-05-059", group: 5 },
  { id: 60, label: "row-05-060", group: 6 },
  { id: 61, label: "row-05-061", group: 7 },
  { id: 62, label: "row-05-062", group: 8 },
  { id: 63, label: "row-05-063", group: 0 },
  { id: 64, label: "row-05-064", group: 1 },
  { id: 65, label: "row-05-065", group: 2 },
  { id: 66, label: "row-05-066", group: 3 },
  { id: 67, label: "row-05-067", group: 4 },
  { id: 68, label: "row-05-068", group: 5 },
  { id: 69, label: "row-05-069", group: 6 },
  { id: 70, label: "row-05-070", group: 7 },
  { id: 71, label: "row-05-071", group: 8 },
  { id: 72, label: "row-05-072", group: 0 },
  { id: 73, label: "row-05-073", group: 1 },
  { id: 74, label: "row-05-074", group: 2 },
  { id: 75, label: "row-05-075", group: 3 },
  { id: 76, label: "row-05-076", group: 4 },
  { id: 77, label: "row-05-077", group: 5 },
  { id: 78, label: "row-05-078", group: 6 },
  { id: 79, label: "row-05-079", group: 7 },
  { id: 80, label: "row-05-080", group: 8 },
  { id: 81, label: "row-05-081", group: 0 },
  { id: 82, label: "row-05-082", group: 1 },
  { id: 83, label: "row-05-083", group: 2 },
  { id: 84, label: "row-05-084", group: 3 },
  { id: 85, label: "row-05-085", group: 4 },
  { id: 86, label: "row-05-086", group: 5 },
  { id: 87, label: "row-05-087", group: 6 },
  { id: 88, label: "row-05-088", group: 7 },
  { id: 89, label: "row-05-089", group: 8 },
  { id: 90, label: "row-05-090", group: 0 },
  { id: 91, label: "row-05-091", group: 1 },
  { id: 92, label: "row-05-092", group: 2 },
  { id: 93, label: "row-05-093", group: 3 },
  { id: 94, label: "row-05-094", group: 4 },
  { id: 95, label: "row-05-095", group: 5 }
];

export function Component05() {
  prepareQueries05();
  return <section data-component="05">
    <h2>Projection component 05</h2>
    <ul>{rows05.map((row) => (
      <li key={row.id} data-group={row.group}>{row.label}</li>
    ))}</ul>
  </section>;
}
