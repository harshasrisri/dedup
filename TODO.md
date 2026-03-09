# Dedup Development Roadmap

Ordered by dependency and priority. Update status as items are completed.

## Active Work Queue

1. [ ] Move all chksum calculations into tokio::spawn_blocking() blocks
   - Wrap synchronous hashing in spawn_blocking to avoid blocking async runtime
   - Update hasher.rs integration points in analyze.rs and local.rs
   - Test with buffer_unordered to ensure no regressions

2. [ ] Break down analyze into frontend (file I/O) and backend (in-mem file_map)
   - Extract backend logic that produces in-mem HashMap<u64, HashSet<String>>
   - Keep frontend handling file I/O and writing output
   - Reusable for other modes

3. [ ] Break down remote into frontend (file I/O) and backend (in-mem file_map)
   - Similar separation: frontend reads/parses file_map files, backend processes in-mem
   - Maintain remote dedup logic

4. [ ] Refactor local to use analyze/remote in-mem file_map capabilities
   - Compose backends from analyze/remote instead of duplicating logic
   - Keep flexibility for future enhancements

5. [ ] Implement persistent file_map in FileStateManager
   - Create $XDG_CACHE_HOME/dedup/state singleton cache
   - Store as global HashMap<PathBuf, HashMap<u64, HashSet<String>>>
   - Use bincode for serialization

## Robustness (Phase 2)

6. [ ] Implement KV state serialization/deserialization with bincode
   - Add bincode as dependency
   - Handle serde encode/decode

7. [ ] Implement file locking mechanism for single-writer guarantee
   - Use fs2 or similar crate for advisory locking
   - Ensure concurrent runs don't corrupt state

8. [ ] Implement atomic write with temp file + rename pattern
   - Write to temp file first
   - Atomic rename on success

9. [ ] Add corruption recovery (warn + rebuild from scratch)
   - Detect bincode decode errors
   - Log warning and rebuild cache from scratch
