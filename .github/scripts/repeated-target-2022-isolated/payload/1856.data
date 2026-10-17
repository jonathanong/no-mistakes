export function readWith() {
  return '/* with/read */ WITH source AS (SELECT id FROM posts) SELECT id FROM source'
}

export function writeWith() {
  return '/* with/write */ WITH changed AS (DELETE FROM posts RETURNING id) SELECT id FROM changed'
}
