declare const index: number;

`SELECT $${1.5}, $${1e21}, $${"12"}`;
`SELECT $${10}, $${1e2}, $${index}`;
`SELECT $${`12`}, $${String.raw`12`}, $${true}, $${null}`;
`SELECT $${-1.5}, $${-1}, $${+1.5}, $${+10}, $${-index}`;
