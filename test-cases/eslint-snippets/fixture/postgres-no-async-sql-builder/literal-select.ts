async function build() {
  return "SELECT 1";
}

function read(query: string): string {
  return query;
}

read(build());
