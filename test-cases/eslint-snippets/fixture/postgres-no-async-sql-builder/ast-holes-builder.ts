const [, value] = [, "SELECT 1"];
// The type after the hole must still be collected; traversal cannot stop early.
type Query = string;
async function build(): Query {
  return "SELECT 1";
}
void build();
