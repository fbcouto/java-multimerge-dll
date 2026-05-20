# 🚀 Adaptive Parallel Multimerge Sort (Java Native Extension via JNI)

This repository contains the high-performance native extension for the Java Virtual Machine (JVM) using the Multimerge parallel sorting algorithm.

By leveraging JNI (Java Native Interface) and the Rust toolchain, the core engine is compiled into a high-performance Dynamic Link Library (.dll on Windows / .so on Linux). This allows Java applications to offload intensive sorting workloads to multi-threaded hardware with absolute zero-copy memory overhead.

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

It modernizes those multi-merge paradigms by utilizing runtime entropy sampling (an Adaptive Oscillation Heuristic) and Rayon's work-stealing parallel scheduler.

---

## 📊 Performance Benchmarks (10M Integers)
We compared the `Rust-MultiMerge` engine against Java's standard sorting utilities using **JMH (Java Microbenchmark Harness)**.

| Algorithm | Random (ms) | Sorted (ms) | Reverse (ms) | Sawtooth (ms) |
| :--- | :--- | :--- | :--- | :--- |
| **Java Sequential** | 918.2 | 3.6 | 10.3 | 101.8 |
| **Java Parallel** | 258.1 | 3.5 | 9.9 | 44.7 |
| **Rust MultiMerge** | **110.8** | **1.6** | **5.1** | 114.6 |

### Key Takeaways:
1. **High Entropies (Random):** The Rust engine is **~8.3x faster** than Java Sequential and **~2.3x faster** than `Arrays.parallelSort`, effectively utilizing the hardware multicore architecture.
2. **Deterministic Speed:** By using parallel boundary scanning, the Rust engine resolves pre-ordered or reversed datasets in near-instant time, outperforming the JVM's internal heuristics.
3. **Hybrid Strategy:** For complex patterns like "Sawtooth," the engine gracefully delegates to the `Rayon` work-stealing scheduler, ensuring stability across all data distributions.

## 🛠️ Engineering Highlights
- **Zero-Copy Architecture:** Accesses JVM heap directly via `get_array_elements_critical`.
- **Rust Core:** Memory-safe, high-concurrency logic leveraging `rayon` and `pdqsort`.
- **Overlapping Chunks:** A cache-aligned approach that allows threads to validate boundaries without inter-thread locking.

## 🚀 How to Run
1. **Compile Rust DLL:** `cargo build --release`
2. **Build Java Benchmarks:** `mvn clean package`
3. **Execute:** ` java "-Djava.library.path=target\release" -jar target\benchmarks.jar`

---


## 1. Prerequisites

Ensure your local environment has the following toolchains installed:

- **Rust Toolchain:** Stable channel (`cargo`, `rustc >= 1.70`)
- **JDK:** Version 8 or higher (`javac`, `java`)
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
│   │               └── JmhSortBenchmark.java   <-- Seu Java vai AQUI
│   └── lib.rs                                  <-- Seu Rust continua AQUI
├── pom.xml
├── Cargo.toml
```

---

# 📜 License

This project is licensed under the Apache License, Version 2.0.