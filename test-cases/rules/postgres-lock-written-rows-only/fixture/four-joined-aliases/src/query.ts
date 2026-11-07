import { beginTransaction } from "@example/db";

export async function lockAndWriteEveryRelation(id: string) {
  const tx = await beginTransaction();
  await tx.query(
    `SELECT n.id FROM notices n JOIN tasks t ON t.notice_id = n.id
     JOIN reviews r ON r.task_id = t.id JOIN decisions d ON d.review_id = r.id
     WHERE n.id = $1 AND t.notice_id = $1 AND r.task_id = $1 AND d.review_id = $1
     FOR UPDATE OF n, t, r, d`,
    [id],
  );
  await tx.query(`UPDATE notices SET seen = true WHERE id = $1`, [id]);
  await tx.query(`UPDATE tasks SET status = 'done' WHERE notice_id = $1`, [id]);
  await tx.query(`UPDATE reviews SET status = 'done' WHERE task_id = $1`, [id]);
  await tx.query(`DELETE FROM decisions WHERE review_id = $1`, [id]);
}

export async function lockAndWriteOnlyPartOfTheRelations(id: string) {
  const tx = await beginTransaction();
  await tx.query(
    `SELECT n.id FROM notices n JOIN tasks t ON t.notice_id = n.id
     JOIN reviews r ON r.task_id = t.id JOIN decisions d ON d.review_id = r.id
     WHERE n.id = $1 AND t.notice_id = $1 AND r.task_id = $1 AND d.review_id = $1
     FOR UPDATE OF n, t, r, d`,
    [id],
  );
  await tx.query(`UPDATE notices SET seen = true WHERE id = $1`, [id]);
  await tx.query(`DELETE FROM tasks WHERE notice_id = $1`, [id]);
}

export async function unknownLockAlias(id: string) {
  const tx = await beginTransaction();
  await tx.query(
    `SELECT n.id FROM notices n JOIN tasks t ON t.notice_id = n.id
     JOIN reviews r ON r.task_id = t.id JOIN decisions d ON d.review_id = r.id
     WHERE n.id = $1 AND t.notice_id = $1 AND r.task_id = $1 AND d.review_id = $1
     FOR UPDATE OF n, t, r, missing`,
    [id],
  );
  await tx.query(`UPDATE notices SET seen = true WHERE id = $1`, [id]);
}
