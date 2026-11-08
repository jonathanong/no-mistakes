import { sql } from "@example/db";

export function columns() {
  return sql`(SELECT p.created_by_id FROM image_placements p WHERE p.placement_id = target.placement_id) AS creator,
    (SELECT p.created_by_id FROM image_placements p WHERE p.placement_id = target.placement_id) AS owner,
    (SELECT p.created_by_id FROM image_placements p WHERE p.placement_id = target.placement_id) AS author,
    (SELECT p.created_by_id FROM image_placements p WHERE p.placement_id = target.placement_id) AS editor`;
}
