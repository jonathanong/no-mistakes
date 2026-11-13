import type { TransactionQuery } from './database.mts';
import { STAGES_00 } from './stages-00.mts';
import { STAGES_01 } from './stages-01.mts';
import { STAGES_02 } from './stages-02.mts';
import { STAGES_03 } from './stages-03.mts';
import { STAGES_04 } from './stages-04.mts';
import { STAGES_05 } from './stages-05.mts';
import { STAGES_06 } from './stages-06.mts';
import { STAGES_07 } from './stages-07.mts';
import { STAGES_08 } from './stages-08.mts';
import { STAGES_09 } from './stages-09.mts';
import { STAGES_10 } from './stages-10.mts';
import { STAGES_11 } from './stages-11.mts';
import { STAGES_12 } from './stages-12.mts';
import { STAGES_13 } from './stages-13.mts';
import { STAGES_14 } from './stages-14.mts';
import { STAGES_15 } from './stages-15.mts';
import { STAGES_16 } from './stages-16.mts';
import { STAGES_17 } from './stages-17.mts';
import { STAGES_18 } from './stages-18.mts';
import { STAGES_19 } from './stages-19.mts';
import { STAGES_20 } from './stages-20.mts';
import { STAGES_21 } from './stages-21.mts';
import { STAGES_22 } from './stages-22.mts';
import { STAGES_23 } from './stages-23.mts';
export async function cleanup(query: TransactionQuery, id: string, n: number) {
  for (const stage of STAGES_00) {
    await stage.run(query, id, n);
    await stage.check(query, id);
  }
  for (const stage of STAGES_01) {
    await stage.run(query, id, n);
    await stage.check(query, id);
  }
  for (const stage of STAGES_02) {
    await stage.run(query, id, n);
    await stage.check(query, id);
  }
  for (const stage of STAGES_03) {
    await stage.run(query, id, n);
    await stage.check(query, id);
  }
  for (const stage of STAGES_04) {
    await stage.run(query, id, n);
    await stage.check(query, id);
  }
  for (const stage of STAGES_05) {
    await stage.run(query, id, n);
    await stage.check(query, id);
  }
  for (const stage of STAGES_06) {
    await stage.run(query, id, n);
    await stage.check(query, id);
  }
  for (const stage of STAGES_07) {
    await stage.run(query, id, n);
    await stage.check(query, id);
  }
  for (const stage of STAGES_08) {
    await stage.run(query, id, n);
    await stage.check(query, id);
  }
  for (const stage of STAGES_09) {
    await stage.run(query, id, n);
    await stage.check(query, id);
  }
  for (const stage of STAGES_10) {
    await stage.run(query, id, n);
    await stage.check(query, id);
  }
  for (const stage of STAGES_11) {
    await stage.run(query, id, n);
    await stage.check(query, id);
  }
  for (const stage of STAGES_12) {
    await stage.run(query, id, n);
    await stage.check(query, id);
  }
  for (const stage of STAGES_13) {
    await stage.run(query, id, n);
    await stage.check(query, id);
  }
  for (const stage of STAGES_14) {
    await stage.run(query, id, n);
    await stage.check(query, id);
  }
  for (const stage of STAGES_15) {
    await stage.run(query, id, n);
    await stage.check(query, id);
  }
  for (const stage of STAGES_16) {
    await stage.run(query, id, n);
    await stage.check(query, id);
  }
  for (const stage of STAGES_17) {
    await stage.run(query, id, n);
    await stage.check(query, id);
  }
  for (const stage of STAGES_18) {
    await stage.run(query, id, n);
    await stage.check(query, id);
  }
  for (const stage of STAGES_19) {
    await stage.run(query, id, n);
    await stage.check(query, id);
  }
  for (const stage of STAGES_20) {
    await stage.run(query, id, n);
    await stage.check(query, id);
  }
  for (const stage of STAGES_21) {
    await stage.run(query, id, n);
    await stage.check(query, id);
  }
  for (const stage of STAGES_22) {
    await stage.run(query, id, n);
    await stage.check(query, id);
  }
  for (const stage of STAGES_23) {
    await stage.run(query, id, n);
    await stage.check(query, id);
  }
}
