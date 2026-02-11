# television vs fzf -- Throughput Benchmark

**System:** Linux 6.17.0-14-generic, x86_64, 20 cores, 30 GB RAM
**tv:** 0.15.2+ (release build, with `5eaa2c3` and `5019651` reverted) | **fzf:** 0.67.0

## Method

Both tools are measured end-to-end: a source command feeds lines into the
tool, which ingests everything and then exits.

- **tv**: `--source-command 'cat <file>' --take-1` (loads all entries, exits)
- **fzf**: `--filter '' --sync` (loads all input with empty query, waits for completion)
- Both wrapped in `script -qc '...' /dev/null` to provide a pseudo-TTY
- 3 runs per scenario, median reported
- ANSI tests use `rg --color=always` as source and `--ansi` on both tools

## Results

### Real-world: `rg` over the hyperscan C++ codebase

Source: [hyperscan](https://github.com/intel/hyperscan) (882 files, 262K
non-empty LOC). Data produced with `rg -n '.'` (all non-empty lines with file
paths and line numbers, ~117 B/line avg).

**Plain text (`rg -n '.'`)**

| Scale |  Lines |   Size | tv (MB/s) | fzf (MB/s) | Speedup |
| ----- | -----: | -----: | --------: | ---------: | ------: |
| 1x    |   262K |  29 MB |   **148** |        123 |    1.2x |
| 10x   |  2.6M | 292 MB |   **444** |        155 |    2.9x |

**ANSI color (`rg -n --color=always '.'` + `--ansi`)**

| Scale |  Lines |   Size | tv (MB/s) | fzf (MB/s) | Speedup |
| ----- | -----: | -----: | --------: | ---------: | ------: |
| 1x    |   262K |  40 MB |   **205** |         98 |    2.1x |
| 10x   |  2.6M | 400 MB |   **481** |        108 |    4.5x |

**ANSI overhead**

| Tool | Plain (10x) | ANSI (10x) | Slowdown |
| ---- | ----------: | ---------: | -------: |
| tv   |   444 MB/s  |  481 MB/s  |     none |
| fzf  |   155 MB/s  |  108 MB/s  |    ~1.4x |

tv shows no throughput penalty from ANSI parsing. fzf slows down ~1.4x.

**Live source command (`rg` as a subprocess, not pre-cached)**

| Mode                       |  Lines |   Size | tv (MB/s) | fzf (MB/s) | Speedup |
| -------------------------- | -----: | -----: | --------: | ---------: | ------: |
| `rg -n '.'`               |   262K |  29 MB |   **159** |        124 |    1.3x |
| `rg -n --color=always '.'` |   262K |  40 MB |   **149** |         96 |    1.6x |

tv is consistently faster across all scenarios, with the gap widening at
higher volumes (2.9x plain, 4.5x ANSI at 10x scale).

## How to reproduce

Any git repository can be used in place of hyperscan. The steps below
are self-contained and only require `tv` (release build), `fzf`, and `rg`.

### 1. Setup

```bash
# Build tv
cargo build --release
TV="./target/release/tv"

# Ensure fzf and rg are available
fzf --version   # tested with 0.67.0
rg --version    # tested with 14.1.1
```

### 2. Pick a target repository

```bash
REPO=~/code/cpp/hyperscan   # any repo works
```

### 3. Generate datasets

```bash
# Plain text
rg -n '.' "$REPO" > /tmp/tv_bench_plain.txt 2>/dev/null

# ANSI color
rg -n --color=always '.' "$REPO" > /tmp/tv_bench_color.txt 2>/dev/null

# Scale x10 for large-volume measurement
for i in $(seq 1 10); do cat /tmp/tv_bench_plain.txt; done > /tmp/tv_bench_plain_10x.txt
for i in $(seq 1 10); do cat /tmp/tv_bench_color.txt; done > /tmp/tv_bench_color_10x.txt

# Verify
wc -l -c /tmp/tv_bench_plain.txt /tmp/tv_bench_color.txt \
         /tmp/tv_bench_plain_10x.txt /tmp/tv_bench_color_10x.txt
```

### 4. Run benchmarks

```bash
bench() {
  local label="$1" file="$2" tv_extra="$3" fzf_extra="$4"
  local bytes=$(wc -c < "$file")
  local lines=$(wc -l < "$file")

  echo "=== $label ($lines lines, $(( bytes / 1048576 )) MB) ==="
  for tool in tv fzf; do
    for i in 1 2 3; do
      START=$(date +%s%N)
      if [ "$tool" = "tv" ]; then
        script -qc "$TV --source-command 'cat $file' $tv_extra --take-1" \
          /dev/null 2>/dev/null >/dev/null
      else
        script -qc "cat $file | fzf $fzf_extra --filter '' --sync > /dev/null" \
          /dev/null 2>/dev/null >/dev/null
      fi
      END=$(date +%s%N)
      MS=$(( (END - START) / 1000000 ))
      MBS=$(echo "scale=1; $bytes / 1048576 * 1000 / $MS" | bc)
      echo "  $tool run $i: ${MS}ms  ${MBS} MB/s"
    done
  done
  echo ""
}

# Plain text
bench "plain 1x"  /tmp/tv_bench_plain.txt     ""       ""
bench "plain 10x" /tmp/tv_bench_plain_10x.txt ""       ""

# ANSI color
bench "ansi 1x"   /tmp/tv_bench_color.txt     "--ansi" "--ansi"
bench "ansi 10x"  /tmp/tv_bench_color_10x.txt "--ansi" "--ansi"

# Live source command
echo "=== live: rg plain ==="
for tool in tv fzf; do
  for i in 1 2 3; do
    START=$(date +%s%N)
    if [ "$tool" = "tv" ]; then
      script -qc "$TV --source-command \"rg -n '.' $REPO\" --take-1" \
        /dev/null 2>/dev/null >/dev/null
    else
      script -qc "rg -n '.' $REPO | fzf --filter '' --sync > /dev/null" \
        /dev/null 2>/dev/null >/dev/null
    fi
    END=$(date +%s%N)
    MS=$(( (END - START) / 1000000 ))
    echo "  $tool run $i: ${MS}ms"
  done
done

echo ""
echo "=== live: rg --color=always + --ansi ==="
for tool in tv fzf; do
  for i in 1 2 3; do
    START=$(date +%s%N)
    if [ "$tool" = "tv" ]; then
      script -qc "$TV --source-command \"rg -n --color=always '.' $REPO\" --ansi --take-1" \
        /dev/null 2>/dev/null >/dev/null
    else
      script -qc "rg -n --color=always '.' $REPO | fzf --ansi --filter '' --sync > /dev/null" \
        /dev/null 2>/dev/null >/dev/null
    fi
    END=$(date +%s%N)
    MS=$(( (END - START) / 1000000 ))
    echo "  $tool run $i: ${MS}ms"
  done
done
```

### 5. Cleanup

```bash
rm -f /tmp/tv_bench_plain.txt /tmp/tv_bench_plain_10x.txt
rm -f /tmp/tv_bench_color.txt /tmp/tv_bench_color_10x.txt
```

## Notes

- `cat` baseline is 5-11 GB/s (page cache), so the source command is not the bottleneck
- tv is ~1.2-1.3x faster at 1x scale and ~2.9-4.5x faster at 10x where startup overhead is amortized
- ANSI parsing has no measurable cost in tv but slows fzf by ~1.4x
- Live `rg` as a source command matches pre-cached throughput
