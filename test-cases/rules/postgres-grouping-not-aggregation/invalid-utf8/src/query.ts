// stray ÿ
import { query } from "db";
export const rows = () =>
  query(`SELECT DISTINCT post_id FROM post_topics WHERE topic_id = $1`);
