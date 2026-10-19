// Casing alone is not a configured tag. The default import below is from an unconfigured package.
import Sql from '@other/sql';
SQL`/* uppercase */ SELECT 1`;
sQl`/* mixed case */ SELECT 1`;
Sql`/* untrusted import */ SELECT 1`;
