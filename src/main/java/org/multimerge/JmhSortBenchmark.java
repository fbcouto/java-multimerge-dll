package org.multimerge;

import java.util.Arrays;
import java.util.Random;
import java.util.concurrent.TimeUnit;

import org.openjdk.jmh.annotations.Benchmark;
import org.openjdk.jmh.annotations.BenchmarkMode;
import org.openjdk.jmh.annotations.Fork;
import org.openjdk.jmh.annotations.Level;
import org.openjdk.jmh.annotations.Measurement;
import org.openjdk.jmh.annotations.Mode;
import org.openjdk.jmh.annotations.OutputTimeUnit;
import org.openjdk.jmh.annotations.Param;
import org.openjdk.jmh.annotations.Scope;
import org.openjdk.jmh.annotations.Setup;
import org.openjdk.jmh.annotations.State;
import org.openjdk.jmh.annotations.Warmup;

@BenchmarkMode(Mode.AverageTime)
@OutputTimeUnit(TimeUnit.MILLISECONDS)
@State(Scope.Thread)
@Warmup(iterations = 3, time = 2)     // 3 ciclos de aquecimento (JIT Compiler)
@Measurement(iterations = 5, time = 2) // 5 ciclos de medição real
@Fork(1)                              // Isola o teste em uma nova JVM
public class JmhSortBenchmark {

    // Carrega a DLL/SO do Rust
    static {
        try {
            System.loadLibrary("rust_multimerge");
        } catch (UnsatisfiedLinkError e) {
            System.err.println("❌ Falha ao carregar a biblioteca nativa. Verifique o java.library.path: " + e.getMessage());
            System.exit(1);
        }
    }

    public static native void multiMergeRust(int[] array);

    // O JMH vai rodar cruzando estes parâmetros (2 tamanhos x 4 distribuições = 8 cenários)
    @Param({"1000000", "10000000"})
    private int size;

    @Param({"RANDOM", "SORTED", "REVERSE", "SAWTOOTH"})
    private String distribution;

    private int[] originalArray;
    private int[] arrayToSort;

    // Roda UMA VEZ por cenário para preparar a massa de dados
    @Setup(Level.Trial)
    public void setupTrial() {
        originalArray = new int[size];
        Random rand = new Random(42);

        for (int i = 0; i < size; i++) {
            originalArray[i] = rand.nextInt();
        }

        switch (distribution) {
            case "RANDOM":
                break; // Já está aleatório
            case "SORTED":
                Arrays.sort(originalArray);
                break;
            case "REVERSE":
                Arrays.sort(originalArray);
                for (int i = 0; i < size / 2; i++) {
                    int temp = originalArray[i];
                    originalArray[i] = originalArray[size - 1 - i];
                    originalArray[size - 1 - i] = temp;
                }
                break;
            case "SAWTOOTH":
                Arrays.sort(originalArray);
                int dentes = 1000;
                int tamanhoChunk = size / dentes;
                if (tamanhoChunk > 1) {
                    for (int d = 0; d < dentes; d++) {
                        if (d % 2 == 1) {
                            int inicio = d * tamanhoChunk;
                            int fim = Math.min(inicio + tamanhoChunk, size);
                            for (int i = 0; i < (fim - inicio) / 2; i++) {
                                int temp = originalArray[inicio + i];
                                originalArray[inicio + i] = originalArray[fim - 1 - i];
                                originalArray[fim - 1 - i] = temp;
                            }
                        }
                    }
                }
                break;
        }
    }

    // Roda ANTES DE CADA invocação do @Benchmark para restaurar a desordem do array
    @Setup(Level.Invocation)
    public void setupInvocation() {
        arrayToSort = originalArray.clone();
    }

    // ==========================================
    // OS COMPETIDORES
    // ==========================================

    @Benchmark
    public void javaParallelSort() {
        Arrays.parallelSort(arrayToSort);
    }

    @Benchmark
    public void rustMultiMergeSort() {
        multiMergeRust(arrayToSort);
    }
    
    @Benchmark
    public void javaSequentialSort() {
    Arrays.sort(arrayToSort);
    }
}