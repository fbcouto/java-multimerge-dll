package org.multimerge;

import java.util.Arrays;
import java.util.Random;

/**
 * Exemplo mínimo de uso da DLL Rust por JNI, mais um autoteste.
 *
 * <pre>
 * javac -d target/classes src/main/java/org/multimerge/Main.java
 * java --enable-native-access=ALL-UNNAMED "-Djava.library.path=target/release" -cp target/classes org.multimerge.Main
 * </pre>
 *
 * Saída esperada: {@code 1 2 5 8 9}, seguida de uma linha OK por distribuição,
 * tamanho e motor. O autoteste compara os dois motores Rust com
 * {@code Arrays.sort} e {@code Arrays.parallelSort}, elemento a elemento.
 * Com o argumento {@code --no-check} só o exemplo é executado; outros
 * argumentos numéricos são tamanhos extras (ex.: {@code 10000000 30000000}).
 */
public class Main {

    static {
        System.loadLibrary("rust_multimerge");
    }

    /** Multimerge: não estável, o padrão. */
    public static native void multiMergeRust(int[] array);

    /** Multimerge_Stable: estável. Com int[] o resultado é o mesmo; muda só o tempo. */
    public static native void multiMergeRustStable(int[] array);

    public static void main(String[] args) {
        int[] data = {5, 2, 9, 1, 8};
        multiMergeRust(data);
        StringBuilder sb = new StringBuilder();
        for (int v : data) {
            if (sb.length() > 0) sb.append(' ');
            sb.append(v);
        }
        System.out.println(sb);

        if (args.length > 0 && args[0].equals("--no-check")) return;

        int[] sizes = {5, 4096, 1_000_000};
        if (args.length > 0) {
            sizes = new int[args.length];
            for (int i = 0; i < args.length; i++) sizes[i] = Integer.parseInt(args[i].replace("_", ""));
        }

        boolean ok = true;
        for (String dist : new String[] {"RANDOM", "SORTED", "REVERSE", "SAWTOOTH"}) {
            for (int size : sizes) {
                ok &= check(dist, size, false);
                ok &= check(dist, size, true);
            }
        }
        if (!ok) {
            System.out.println("FALHOU");
            System.exit(1);
        }
    }

    private static boolean check(String dist, int size, boolean stable) {
        int[] original = generate(dist, size);
        int[] expected = original.clone();
        Arrays.sort(expected);
        int[] parallel = original.clone();
        Arrays.parallelSort(parallel);
        int[] actual = original.clone();
        if (stable) multiMergeRustStable(actual); else multiMergeRust(actual);
        boolean okSeq = Arrays.equals(expected, actual);
        boolean okPar = Arrays.equals(parallel, actual);
        boolean ok = okSeq && okPar;
        System.out.printf("%-4s %-8s %9d %-17s (== Arrays.sort: %s, == Arrays.parallelSort: %s)%n",
                ok ? "OK" : "FAIL", dist, size, stable ? "Multimerge_Stable" : "Multimerge",
                okSeq, okPar);
        return ok;
    }

    /** Mesmas distribuições do JmhSortBenchmark. */
    private static int[] generate(String dist, int size) {
        int[] a = new int[size];
        Random rnd = new Random(42);
        switch (dist) {
            case "RANDOM":
                for (int i = 0; i < size; i++) a[i] = rnd.nextInt();
                break;
            case "SORTED":
                for (int i = 0; i < size; i++) a[i] = i;
                break;
            case "REVERSE":
                for (int i = 0; i < size; i++) a[i] = size - i;
                break;
            case "SAWTOOTH":
                int block = Math.max(1, size / 10);
                for (int i = 0; i < size; i++) a[i] = i;
                for (int start = block; start < size; start += 2 * block) {
                    int end = Math.min(start + block, size) - 1;
                    for (int i = start, j = end; i < j; i++, j--) {
                        int t = a[i]; a[i] = a[j]; a[j] = t;
                    }
                }
                break;
            default:
                throw new IllegalArgumentException(dist);
        }
        return a;
    }
}
