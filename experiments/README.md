# Experiments

Standalone experiment scripts and test crates for WDRProbe development.
**Not part of the Tauri application build.** These are development-only exploration tools.

## Contents

### `wdr_parser_test/` — WDR HTML Parser Experiment

Standalone crate that tests WDR HTML parsing using the `scraper` crate.
Entry point: `wdr_parser_main.rs`

```bash
cd experiments/wdr_parser_test
cargo run
```

### `cache_io_test/` — Cache I/O Performance Test

Standalone crate for I/O performance benchmarking.

```bash
cd experiments/cache_io_test
cargo run
```

### `scripts/` — One-off Test Utilities

Independent scripts (each with `fn main()`) for testing specific WDR parsing features.
These are not part of any crate — compile and run individually:

```bash
rustc experiments/scripts/test_cache_io.rs && ./test_cache_io
rustc experiments/scripts/test_wdr_parser.rs && ./test_wdr_parser
rustc experiments/scripts/wdr_complete_parser_test.rs && ./wdr_complete_parser_test
```

**Note:** These scripts contain hardcoded absolute file paths and will not work
without modification. They are preserved for reference only.

---

⚠️ **These are standalone experiments, not part of the WDRProbe application.**
For the actual app tests, see `Desktop/src-tauri/tests/`.
