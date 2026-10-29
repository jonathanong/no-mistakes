import { read } from '@bench/db'
import sql from 'sql-template-strings'
import { buildPrivacyFilter } from './privacy-filter.mts'
import type { Viewer } from './privacy-filter.mts'

export function readVisiblePosts(viewer: Viewer | null): void {
  const query = sql`/* readVisiblePosts */ SELECT id FROM posts WHERE`
  // Protected invariant: the helper may add only a source-visible predicate; the named head stays fixed.
  const filter = buildPrivacyFilter('posts', viewer)
  if (filter) query.append(sql` `).append(filter)
  else query.append(sql` TRUE`)
  read(query)
}

export function readPostsWithUnsafeAlias(viewer: Viewer | null, alias: string): void {
  const query = sql`/* readPostsWithUnsafeAlias */ SELECT id FROM posts WHERE`
  // Unsafe control: an arbitrary alias can change the SQL fragment before append.
  const filter = buildPrivacyFilter(alias, viewer)
  if (filter) query.append(sql` `).append(filter)
  else query.append(sql` TRUE`)
  read(query)
}

export function readOverwrittenHead(viewer: Viewer | null, text: string): void {
  const query = sql`/* overwritten */ SELECT id FROM posts`
  // A complete SQL proof and an annotation proof must both reject this write.
  query.text = text
  read(query) // low: false
}
