import { read } from "@example/db";
import { sql } from "@example/sql";
import {
  visibleImported,
  hiddenImported,
  visibleBagImported,
  hiddenBagImported,
  visibleTaggedImported,
  hiddenTagImported,
  visibleNamedBagImported,
  hiddenCallImported,
  hiddenApplyImported,
} from "./union-helper";

visibleImported(read);
hiddenImported(sql`SELECT ${fragment} FROM t`, read);
visibleBagImported({ query: read });
hiddenBagImported(sql`SELECT ${fragment} FROM t`, { query: read });
visibleTaggedImported(read);
hiddenTagImported(read);
visibleNamedBagImported({ query: read, id });
hiddenCallImported(fragment, { query: read });
hiddenApplyImported(fragment, { query: read });
