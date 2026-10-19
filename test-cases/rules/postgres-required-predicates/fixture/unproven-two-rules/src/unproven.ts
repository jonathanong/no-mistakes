export function u(db: { query(text: string): unknown }) {
  db.query("SELECT id FROM users WHERE x = 9");
}
