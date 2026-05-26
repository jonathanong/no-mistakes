## 2026-05-23 - [Optimize graph builder BFS sort]
**Learning:** Using `sort_by_cached_key` instead of `sort_by_key` for sorting adjacency lists during graph building offers a measurable performance improvement for large repositories (from 33ms to 31ms).
**Action:** Always consider `sort_by_cached_key` when sorting vectors by keys that require calculation or allocation (like `String` creation from an enum), especially on hot paths.
## 2026-05-23 - [Optimize HashSet Insertions with Pre-Checks]
**Learning:** In hot graph traversal loops (like BFS), blindly calling `visited.insert(value.clone())` where the value is an expensive type like `String` or `PathBuf` introduces significant performance overhead due to unnecessary heap allocations for already visited nodes.
**Action:** Always pre-check `!visited.contains(&value)` before performing the expensive `clone()` and `insert()` combination. This pattern avoids redundant memory allocations.
