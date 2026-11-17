// Biblioteca nativa JNI do MultiMerge.
//
// O motor esta em `multimerge.rs`, copia de
// adaptive-parallel-multimerge-sort/src/multimerge.rs. Ele substitui o motor
// que morava neste arquivo (Timsort + merge paralelo). O que muda para quem
// chama do Java:
//
//   multiMergeRust / multiMergeRustStrings              Multimerge, NAO estavel
//   multiMergeRustStable / multiMergeRustStringsStable  Multimerge_Stable
//
// Com `int[]` e com texto de largura fixa, dois elementos iguais sao
// indistinguiveis: os dois motores devolvem exatamente o mesmo array, e a
// diferenca entre eles e so de tempo.
pub mod multimerge;

use jni::objects::{JByteArray, JClass, JIntArray, ReleaseMode};
use jni::JNIEnv;

/// Multimerge: NAO estavel. E o padrao.
pub fn ordenar_multi_merge<T: Ord + Copy + Send + Sync>(arr: &mut [T]) {
    multimerge::multi_merge_sort(arr);
}

/// Multimerge_Stable: preserva a ordem relativa de chaves iguais.
pub fn ordenar_multi_merge_estavel<T: Ord + Copy + Send + Sync>(arr: &mut [T]) {
    multimerge::multi_merge_sort_stable(arr);
}

// ==========================================
// CAMADA DE INTERFACE JNI (JAVA)
// ==========================================

// Estrutura estatica para texto em flat bytes (6 bytes fixos)
#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord)]
#[repr(transparent)]
pub struct StringFixa(pub [u8; 6]);

fn ordenar_ints(env: &mut JNIEnv, input: &JIntArray, estavel: bool) {
    // Ponteiro critico: bloqueia o Garbage Collector e evita a copia.
    let mut elements = unsafe {
        env.get_array_elements_critical(input, ReleaseMode::CopyBack)
            .expect("Erro ao obter elementos do array Java")
    };

    let slice: &mut [i32] = unsafe {
        std::slice::from_raw_parts_mut(elements.as_mut_ptr() as *mut i32, elements.len())
    };

    if estavel {
        ordenar_multi_merge_estavel(slice);
    } else {
        ordenar_multi_merge(slice);
    }
}

fn ordenar_strings(env: &mut JNIEnv, input: &JByteArray, estavel: bool) {
    let mut elements = unsafe {
        env.get_array_elements_critical(input, ReleaseMode::CopyBack)
            .expect("Erro ao obter elementos do array de bytes do Java")
    };

    let slice_u8: &mut [u8] = unsafe {
        std::slice::from_raw_parts_mut(elements.as_mut_ptr() as *mut u8, elements.len())
    };

    let mut vetor_strings: Vec<StringFixa> = slice_u8
        .chunks_exact(6)
        .map(|chunk| {
            let mut bytes = [0u8; 6];
            bytes.copy_from_slice(chunk);
            StringFixa(bytes)
        })
        .collect();

    if estavel {
        ordenar_multi_merge_estavel(&mut vetor_strings);
    } else {
        ordenar_multi_merge(&mut vetor_strings);
    }

    for (i, string_fixa) in vetor_strings.iter().enumerate() {
        let idx = i * 6;
        slice_u8[idx..idx + 6].copy_from_slice(&string_fixa.0);
    }
}

// ---- org.multimerge.JmhSortBenchmark

#[no_mangle]
pub extern "system" fn Java_org_multimerge_JmhSortBenchmark_multiMergeRust<'local>(
    mut env: JNIEnv<'local>,
    _class: JClass<'local>,
    input: JIntArray<'local>,
) {
    ordenar_ints(&mut env, &input, false);
}

#[no_mangle]
pub extern "system" fn Java_org_multimerge_JmhSortBenchmark_multiMergeRustStable<'local>(
    mut env: JNIEnv<'local>,
    _class: JClass<'local>,
    input: JIntArray<'local>,
) {
    ordenar_ints(&mut env, &input, true);
}

#[no_mangle]
pub extern "system" fn Java_org_multimerge_JmhSortBenchmark_multiMergeRustStrings<'local>(
    mut env: JNIEnv<'local>,
    _class: JClass<'local>,
    input: JByteArray<'local>,
) {
    ordenar_strings(&mut env, &input, false);
}

#[no_mangle]
pub extern "system" fn Java_org_multimerge_JmhSortBenchmark_multiMergeRustStringsStable<'local>(
    mut env: JNIEnv<'local>,
    _class: JClass<'local>,
    input: JByteArray<'local>,
) {
    ordenar_strings(&mut env, &input, true);
}

// ---- org.multimerge.Main (teste simples do usuario)

#[no_mangle]
pub extern "system" fn Java_org_multimerge_Main_multiMergeRust<'local>(
    mut env: JNIEnv<'local>,
    _class: JClass<'local>,
    input: JIntArray<'local>,
) {
    ordenar_ints(&mut env, &input, false);
}

#[no_mangle]
pub extern "system" fn Java_org_multimerge_Main_multiMergeRustStable<'local>(
    mut env: JNIEnv<'local>,
    _class: JClass<'local>,
    input: JIntArray<'local>,
) {
    ordenar_ints(&mut env, &input, true);
}

#[cfg(test)]
mod tests {
    use super::*;

    fn lcg(s: &mut u64) -> u64 {
        *s = s.wrapping_mul(6364136223846793005).wrapping_add(1442695040888963407);
        *s >> 11
    }

    /// As quatro distribuicoes do JmhSortBenchmark, nos dois motores.
    #[test]
    fn distribuicoes_do_benchmark() {
        let n = 1_000_000usize;
        let mut s = 42u64;
        let aleatorio: Vec<i32> = (0..n).map(|_| lcg(&mut s) as i32).collect();
        let mut ordenado = aleatorio.clone();
        ordenado.sort();
        let invertido: Vec<i32> = ordenado.iter().rev().copied().collect();
        // SAWTOOTH do benchmark: ordenado, com os blocos impares invertidos.
        let mut serra = ordenado.clone();
        let bloco = n / 1000;
        for d in (1..1000).step_by(2) {
            serra[d * bloco..(d + 1) * bloco].reverse();
        }
        for (nome, dado) in [
            ("RANDOM", &aleatorio),
            ("SORTED", &ordenado),
            ("REVERSE", &invertido),
            ("SAWTOOTH", &serra),
        ] {
            let (mut a, mut b) = (dado.clone(), dado.clone());
            ordenar_multi_merge(&mut a);
            ordenar_multi_merge_estavel(&mut b);
            assert!(a == ordenado, "{nome}: Multimerge");
            assert!(b == ordenado, "{nome}: Multimerge_Stable");
        }
    }

    #[test]
    fn texto_de_largura_fixa() {
        let mut s = 7u64;
        for n in [0usize, 1, 5, 4097, 300_000] {
            let textos: Vec<StringFixa> = (0..n)
                .map(|_| {
                    let mut t = [0u8; 6];
                    for c in t.iter_mut() {
                        *c = b'A' + (lcg(&mut s) % 4) as u8;
                    }
                    StringFixa(t)
                })
                .collect();
            let mut esperado = textos.clone();
            esperado.sort();
            let (mut a, mut b) = (textos.clone(), textos.clone());
            ordenar_multi_merge(&mut a);
            ordenar_multi_merge_estavel(&mut b);
            assert!(a == esperado, "Multimerge, n={n}");
            assert!(b == esperado, "Multimerge_Stable, n={n}");
        }
    }
}
