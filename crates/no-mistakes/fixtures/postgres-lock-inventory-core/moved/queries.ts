import { query } from "@example/db";

void query(`
  SELECT id FROM jobs FOR /* wait */ SHARE
`);

declare function reserveJob(id: number): void;
reserveJob(10); reserveJob(/* id */ 20);
declare function reservejob(id: number): void;
reservejob(3);
