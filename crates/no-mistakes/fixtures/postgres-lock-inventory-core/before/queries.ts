import { query } from "@example/db";

declare function reserveJob(id: number): void;
reserveJob(1);
reserveJob(2);
void query("SELECT id FROM jobs FOR SHARE");
declare function reservejob(id: number): void;
reservejob(3);
