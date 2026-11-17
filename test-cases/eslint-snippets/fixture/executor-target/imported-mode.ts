import { read } from "@example/db";
import { sql } from "@example/sql";
import {
  visibleImported,
  hiddenImported,
  visibleBagImported,
  hiddenBagImported,
  visibleTaggedImported,
  hiddenTaggedFragmentImported,
  hiddenTagImported,
  visibleNamedBagImported,
  mutatedNamedBagImported,
  hiddenCallImported,
  hiddenApplyImported,
} from "./union-helper";

visibleImported(read);
hiddenImported(sql`SELECT ${fragment} FROM t`, read);
visibleBagImported({ query: read });
hiddenBagImported(sql`SELECT ${fragment} FROM t`, { query: read });
visibleTaggedImported(read);
hiddenTaggedFragmentImported(fragment, read);
hiddenTagImported(read);
visibleNamedBagImported({ query: read, id });
mutatedNamedBagImported({ query: read, id: 1 });
hiddenCallImported(fragment, { query: read });
hiddenApplyImported(fragment, { query: read });
