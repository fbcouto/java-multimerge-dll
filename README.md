# 🚀 Adaptive Parallel Multimerge Sort (Java Native Extension via JNI)

This repository contains the high-performance native extension for the Java Virtual Machine (JVM) using the Multimerge parallel sorting algorithm.

By leveraging JNI (Java Native Interface) and the Rust toolchain, the core engine is compiled into a high-performance Dynamic Link Library (.dll on Windows / .so on Linux). This allows Java applications to offload intensive sorting workloads to multi-threaded hardware with absolute zero-copy memory overhead.

The library exposes two engines:

| Java native method | Engine | Stability |
| :--- | :--- | :--- |
| `multiMergeRust(int[])` | **Multimerge** (default) | not stable |
| `multiMergeRustStable(int[])` | **Multimerge_Stable** | stable |

With `int[]` (and fixed-width byte strings) equal elements are indistinguishable, so both engines return the same array; the stable variant exists for callers that sort keys attached to payloads, and the benchmark below measures what its guarantee costs.

---

# 🔗 Core Algorithm & Academic Background

> 📌 **Note:** The mathematical foundations, dynamic heuristics, and exhaustive standalone benchmarks of the Multimerge engine are fully detailed and tested in the primary repository:
>
> 👉 **[Core Multimerge Sorting Repository](https://github.com/fbcouto/adaptive-parallel-multimerge-sort)**

The core theoretical foundation of this parallel architecture is based on the original research and paper:

- **Title:** Multimerge
- **Authors:** Fernando B. Couto & Fábio S. Couto
- **Conference:** PDPTA'11 — The 2011 International Conference on Parallel and Distributed Processing Techniques and Applications
- **Lecture Series:** WorldComp'11 (The 2011 World Congress in Computer Science, Computer Engineering, and Applied Computing)
- The architecture implements a hybrid processing model based on the original *Multimerge* paper published in **PDPTA'11** (The 2011 International Conference on Parallel and Distributed Processing Techniques and Applications).

The engine in this repository (`src/multimerge.rs`) is a verbatim copy of the core repository's engine, rewritten in November 2026. It modernizes the multi-merge paradigm with a cheap pre-pass: it samples the input to estimate the inversion rate, detects the natural ascending and descending runs, reverses the descending ones in place, and merges only what is actually out of order. Runs that already occupy disjoint value ranges are coalesced without a merge. Inputs with no exploitable structure fall through to a parallel sort on Rayon's work-stealing scheduler.

---

## 📊 Performance Benchmarks
Both Rust engines were compared against Java's standard sorting utilities using **JMH (Java Microbenchmark Harness)**, on `int[]` arrays of 1M, 10M and 30M elements in four distributions:

- **Random:** uniform `Random(42).nextInt()`.
- **Sorted:** already ascending.
- **Reverse:** strictly descending.
- **Sawtooth:** sorted, then every odd block out of 1,000 reversed. Each tooth sits in its own disjoint value range.

Average time per sort, in milliseconds; lower is better. The best result in each column is in large bold blue (rendered by GitHub's math extension; other Markdown viewers may show the raw `\color` markup).

### 1M integers

| Algorithm | Random (ms) | Sorted (ms) | Reverse (ms) | Sawtooth (ms) |
| :--- | ---: | ---: | ---: | ---: |
| Java `Arrays.sort` | 89.0 | 0.41 | 1.09 | 11.0 |
| Java `Arrays.parallelSort` | 24.2 | 0.46 | 1.13 | 4.17 |
| Rust Multimerge | ${\color{blue}\Large\textbf{9.94}}$ | ${\color{blue}\Large\textbf{0.31}}$ | ${\color{blue}\Large\textbf{0.71}}$ | ${\color{blue}\Large\textbf{0.56}}$ |
| Rust Multimerge_Stable | 11.7 | 0.32 | 1.41 | 0.57 |

### 10M integers

| Algorithm | Random (ms) | Sorted (ms) | Reverse (ms) | Sawtooth (ms) |
| :--- | ---: | ---: | ---: | ---: |
| Java `Arrays.sort` | 1079.3 | 3.51 | 10.5 | 95.8 |
| Java `Arrays.parallelSort` | 285.1 | 3.75 | 10.5 | 46.7 |
| Rust Multimerge | ${\color{blue}\Large\textbf{113.3}}$ | ${\color{blue}\Large\textbf{1.93}}$ | ${\color{blue}\Large\textbf{7.80}}$ | ${\color{blue}\Large\textbf{4.88}}$ |
| Rust Multimerge_Stable | 129.7 | 1.98 | 11.3 | 5.00 |

### 30M integers

| Algorithm | Random (ms) | Sorted (ms) | Reverse (ms) | Sawtooth (ms) |
| :--- | ---: | ---: | ---: | ---: |
| Java `Arrays.sort` | 3258.3 | 11.4 | 31.6 | 294.3 |
| Java `Arrays.parallelSort` | 907.2 | 11.8 | 32.3 | 149.7 |
| Rust Multimerge | ${\color{blue}\Large\textbf{340.9}}$ | ${\color{blue}\Large\textbf{5.11}}$ | ${\color{blue}\Large\textbf{17.5}}$ | ${\color{blue}\Large\textbf{11.0}}$ |
| Rust Multimerge_Stable | 422.3 | 5.20 | 32.9 | 13.1 |

### Key Takeaways:
1. **Random (no structure):** Multimerge is **2.4x to 2.7x faster** than `Arrays.parallelSort` and **~9x to 9.5x faster** than `Arrays.sort`, at all three sizes. This is the one case where the engine finds nothing to exploit and the gap is pure parallel sorting throughput.
2. **Sawtooth (piecewise order):** this is where the engine pays off most. Multimerge resolves 1,000 alternating ascending and descending teeth in a single pass (reverse the odd teeth, coalesce the disjoint ranges, no merge needed): **7.5x faster** than `Arrays.parallelSort` at 1M, **9.6x** at 10M and **13.6x** at 30M, and **20x to 27x faster** than `Arrays.sort`. The advantage grows with size because the Java sorts still do O(n log n) work on data that is already 99.9% ordered.
3. **Sorted and Reverse:** all four algorithms detect these in roughly linear time; the Rust engines are the fastest at every size (1.3x to 2.3x over `Arrays.sort`), but the absolute times are a few milliseconds and the differences are within noise at 1M.
4. **The price of stability:** `Multimerge_Stable` costs **15% to 24%** more than `Multimerge` on random data (it cannot use the unstable parallel quicksort) and up to 2x on Reverse (it must preserve the order of equal keys while reversing). On Sorted and Sawtooth it costs nothing measurable. With `int[]` both produce the identical array, so pick `Multimerge` unless you sort keys attached to payloads.

**Method.** Intel Core i7-7700HQ (4 cores / 8 threads), Windows 10, JDK 24.0.2, JMH 1.37, 1 fork, 3 warm-up + 5 measurement iterations of 2 s each, a fresh copy of the input before every invocation. The full suite was run twice per size; in each cell the run with the narrower 99.9% confidence interval is shown. Interference (GC, other processes) only ever adds time, so of two disagreeing runs the lower, tighter one is the closer estimate of the algorithm's cost. The two runs agreed within 10% on most cells; the worst outliers were `Arrays.parallelSort` on Sawtooth 10M (161.9 ± 227 ms vs 46.7 ± 3.1 ms) and `Arrays.sort` on Random 30M (3445 ± 2080 vs 3258 ± 941). Expect 15% to 50% variation between identical runs on a laptop; read the ratios, not the third digit. The 30M numbers come from `-p size=30000000` on the command line (see **How to Run**).

## 🛠️ Engineering Highlights
- **Zero-Copy Architecture:** Accesses the JVM heap directly via `get_array_elements_critical`; the `int[]` is sorted in place, no copy in or out.
- **Rust Core:** Memory-safe, high-concurrency logic on `rayon` and the standard library's sorts. The engine file is shared, byte for byte, with the [core repository](https://github.com/fbcouto/adaptive-parallel-multimerge-sort), where its tests live.
- **Structure First:** Before sorting, the engine samples the inversion rate and detects natural runs. Sorted and reversed inputs are resolved in a single pass; piecewise-ordered inputs (such as the sawtooth below) become a handful of segments that are coalesced or merged, never re-sorted from scratch.
- **Two Engines, One Switch:** `Multimerge` (default, unstable) may use a parallel quicksort on chaotic data; `Multimerge_Stable` keeps the relative order of equal keys, at a measurable cost only where the data is actually chaotic.

## 🚀 How to Run
1. **Compile Rust DLL:** `cargo build --release` (produces `target/release/rust_multimerge.dll`)
2. **Build Java Benchmarks:** `mvn clean package` (the `pom.xml` configures `mvn clean` to leave Cargo's output in `target/` alone)
3. **Execute:** `java --enable-native-access=ALL-UNNAMED "-Djava.library.path=target\release" -jar target\benchmarks.jar`

The full run is 32 benchmarks (4 competitors × 2 sizes × 4 distributions), about 10 minutes. To measure another size, override the parameter: `-p size=30000000` (about 8 minutes; the 30M table above was produced this way). To run a single competitor or distribution: `-p distribution=SAWTOOTH rustMultiMergeSort`.

---


## 1. Prerequisites

Ensure your local environment has the following toolchains installed:

- **Rust Toolchain:** Stable channel (`cargo`, `rustc >= 1.70`)
- **JDK:** Version 11 or higher (`javac`, `java`). The `--enable-native-access=ALL-UNNAMED` flag used below exists since JDK 17; on older JDKs just omit it.
- **Maven**
---

## 2. Layout Structure

```text
multimerge-Java-dll/
├── src\
│   ├── main\
│   │   └── java\
│   │       └── org\
│   │           └── multimerge\
│   │               ├── JmhSortBenchmark.java   <-- JMH benchmark (4 competitors)
│   │               └── Main.java               <-- minimal example + self-test
│   ├── lib.rs                                  <-- JNI layer (Rust)
│   └── multimerge.rs                           <-- sorting engine (copy of the core repository)
├── pom.xml
├── Cargo.toml
```

---
## 🚀 Quick Start Example
To verify your installation, you can run the following simple test:

```bash
# Compile and run
javac -d target/classes src/main/java/org/multimerge/Main.java
java --enable-native-access=ALL-UNNAMED "-Djava.library.path=target/release" -cp "target/classes" org.multimerge.Main

# Expected Output:
# 1 2 5 8 9
# OK   RANDOM           5 Multimerge        (== Arrays.sort: true, == Arrays.parallelSort: true)
# OK   RANDOM           5 Multimerge_Stable (== Arrays.sort: true, == Arrays.parallelSort: true)
# ...                                        (24 lines: 4 distributions x 3 sizes x 2 engines)
```

The first line is `{5, 2, 9, 1, 8}` sorted by the native library. The lines that follow are a self-test: the output of both engines is compared element by element with `Arrays.sort` and `Arrays.parallelSort` on the four distributions used by the benchmark, at 5, 4,096 and 1,000,000 elements. Pass `--no-check` to print only the first line, or pass sizes to check other lengths (the benchmark sizes were all verified this way):

```bash
java -Xmx3g --enable-native-access=ALL-UNNAMED "-Djava.library.path=target/release" -cp "target/classes" org.multimerge.Main 10000000 30000000
```
---
# 📄 License

This project is licensed under the Apache License 2.0.

You may obtain a copy of the license at:

- https://www.apache.org/licenses/LICENSE-2.0

Copyright © Fernando B. Couto

Licensed under the Apache License, Version 2.0 (the "License");
you may not use this project except in compliance with the License.
You may obtain a copy of the License at:

http://www.apache.org/licenses/LICENSE-2.0

Unless required by applicable law or agreed to in writing, software
distributed under the License is distributed on an "AS IS" BASIS,
WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
See the License for the specific language governing permissions and
limitations under the License.
