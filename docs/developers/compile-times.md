# Compile times

Notes from a June 2026 investigation into why release builds felt like they
took an eternity, dominated by the final `television` (lib) and `tv` (bin)
compilation units running sequentially on a single core.

All numbers below were measured on an 8-core Linux machine with Rust 1.93,
using `cargo build --timings` on clean builds unless stated otherwise.

## TL;DR

- The release profile used `codegen-units = 1` + `lto = "fat"`, which made the
  entire tail of the build serial. It now uses `codegen-units = 16` +
  `lto = "thin"` + `incremental = true`; a separate `dist` profile keeps the
  maximally-optimized settings for shipped artifacts (CI, `cargo deb`, Nix).
- Independently of profile settings, ~200s of every build is a **single,
  unsplittable ~2.2M-line codegen unit** containing all of frizbee's
  monomorphized SIMD backends. Incremental compilation caches it across
  rebuilds; only structural changes (see [Going further](#going-further)) can
  remove it from clean builds.

| Scenario                                  | Before  | After       |
| ----------------------------------------- | ------- | ----------- |
| Clean `--release` build                   | 8m 00s  | 6m 16s      |
| `--release` rebuild after a code change   | ~6m     | ~2m 05s     |
| `staging` rebuild after a code change     | minutes | **~1.6s**   |
| Shipped artifacts (`dist` profile)        | –       | unchanged   |

## Profile layout

```toml
[profile.release]   # local development: fast-ish optimized builds
codegen-units = 16
lto = "thin"
incremental = true

[profile.dist]      # what CI ships: slow to build, maximally optimized
inherits = "release"
codegen-units = 1
lto = "fat"
incremental = false

[profile.staging]   # `just r`: optimized, no LTO, ~2s incremental rebuilds
inherits = "release"
lto = false
```

`dist` is used by the CD workflow (`--profile dist`), `cargo deb`
(`[profile.deb]` inherits from it), and the Nix flake (`CARGO_PROFILE = "dist"`
via crane). Local `cargo build --release` never pays the fat-LTO cost.

## The frizbee mega-CGU

Even with 16 codegen units, the `television` lib crate spent ~210s in codegen
at ~1.2× CPU parallelism, and raising `codegen-units` to 64 changed nothing.
Emitting per-CGU LLVM IR (`cargo rustc --release --lib -- --emit=llvm-ir`)
showed why: one CGU contained **2.19M lines of IR — 61% of the whole crate**
(8× the next largest), and it was ~100% frizbee `matcher::backend` code.

Three compounding mechanisms produce this:

1. **Monomorphization happens at the call site.** frizbee's `Matcher` methods
   are generic (`match_list_parallel<S: AsRef<str>>`, …), and every layer
   below them keeps a generic parameter alive (`S`/`H`, plus
   `const TYPOS: u16, const UNICODE: bool`). Nothing in that tree can be
   precompiled inside frizbee's own crate; a single call from
   `television::matcher` transitively instantiates all of it — ~15 backends
   (AVX-512/AVX2/SSE/NEON/scalar × u8/u16 × literal) × 10 typo/unicode combos
   ≈ 160 copies, generated inside the `television` crate.

2. **`#[inline(always)]` flattening.** `#[target_feature]` only applies to the
   function it's attached to, so for the Smith-Waterman hot loops to actually
   compile as AVX-512/AVX2/… code, frizbee force-inlines the full
   prefilter + scoring implementation into each `Specialized` backend method.
   Each of the 160 copies is a giant self-contained body of dense vector IR
   (the largest single function we measured was ~268k IR lines).

3. **CGU partitioning is per defining module.** rustc places all
   monomorphizations from the same defining module
   (`frizbee::matcher::backend`) into the same codegen unit, and a module is
   never split across CGUs. So no `codegen-units` value can parallelize those
   2.2M lines: they are ~200s of inherently serial LLVM time.

### Why incremental compilation fixes the rebuild case

Incremental compilation caches compiled objects per CGU. Editing television
code doesn't invalidate the frizbee CGU, so it is reused as-is: lib rebuilds
drop from ~3.5min to ~1.5s. What remains on a release rebuild is the `tv`
bin's thin-LTO link (~2min); the `staging` profile skips LTO entirely and
rebuilds in ~2s.

CI is unaffected: `Swatinem/rust-cache` sets `CARGO_INCREMENTAL=0`, and the
`dist` profile disables incremental explicitly.

## Going further

Clean-build time is now bounded below by the serial frizbee CGU (+ LTO).
Options, in order of preference:

1. **Upstream fix in frizbee**: instantiate the backend hot loops inside the
   frizbee crate (e.g. concrete `&[&str]` entry points behind thin generic
   shims). The cost would then be paid inside frizbee's own compilation —
   early, in parallel with the other ~400 dependency units, and cached until
   the dependency is bumped. Benefits every frizbee user.
2. **Local shim crate**: a tiny workspace crate exposing non-generic wrappers
   around the three frizbee calls television makes (`Matcher::from_query`,
   `match_list_parallel`, `match_one_indices`). Moves the instantiation root
   into a crate that almost never gets rebuilt, achieving the same caching and
   letting cargo's pipelining overlap the 200s with the rest of a clean build.
   Cost: television is published to crates.io, so the shim would need to be
   published and version-bumped in lockstep. Less compelling now that
   incremental compilation covers the rebuild case.

## Useful diagnostics

```sh
cargo build --release --timings        # per-crate timeline (target/cargo-timings/)
cargo llvm-lines --release --lib       # IR lines per function (monomorphization bloat)
cargo rustc --release --lib -- --emit=llvm-ir   # per-CGU .ll files in target/release/deps/
```

For the timings HTML, the interesting fields are each unit's total duration vs
its `rmeta` time (frontend); the gap is codegen. A large gap with low CPU
utilization (`user` ≈ `real` in `time`) indicates one dominant CGU.
