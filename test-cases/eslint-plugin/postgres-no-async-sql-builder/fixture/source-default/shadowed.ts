import sql from "@example/db";
function run(sql: (text: TemplateStringsArray) => string) {
  async function build() { return sql`SELECT 1`; }
  void build();
}
