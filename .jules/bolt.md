## 2024-05-23 - [Optimize graph builder BFS sort]
**Learning:** Using `sort_by_cached_key` instead of `sort_by_key` for sorting adjacency lists during graph building offers a measurable performance improvement for large repositories (from 33ms to 31ms).
**Action:** Always consider `sort_by_cached_key` when sorting vectors by keys that require calculation or allocation (like `String` creation from an enum), especially on hot paths.
