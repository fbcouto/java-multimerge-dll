use rayon::prelude::*;

// ==========================================
// 1. UTILITÁRIOS GENÉRICOS
// ==========================================

fn calcular_minrun(mut n: usize) -> usize {
    let mut r = 0;
    while n >= 64 {
        r |= n & 1;
        n >>= 1;
    }
    n + r
}

fn detectar_tendencia_global<T: Ord + Sync>(arr: &mut [T]) -> bool {
    let n = arr.len();
    if n < 100_000 { return false; } 

    let chunk_size = 32768; 
    let qtd_blocos = (n + chunk_size - 1) / chunk_size;
    let arr_imutavel: &[T] = arr;

    // ========================================================
    // O MAP-REDUCE: Agora rastreia tanto subidas quanto descidas
    // ========================================================
    let (subindo, descendo) = (0..qtd_blocos)
        .into_par_iter()
        .map(|i| {
            let inicio = i * chunk_size;
            let fim = std::cmp::min(inicio + chunk_size + 1, n);
            let chunk_sobreposto = &arr_imutavel[inicio..fim];

            let mut asc = 0;
            let mut desc = 0;

            for j in 1..chunk_sobreposto.len() {
                if chunk_sobreposto[j - 1] < chunk_sobreposto[j] {
                    asc += 1; // Degrau subindo
                } else if chunk_sobreposto[j - 1] > chunk_sobreposto[j] {
                    desc += 1; // Degrau descendo
                }
            }
            
            // A thread retorna uma tupla com as duas contagens
            (asc, desc) 
        })
        // O Reduce soma as tuplas de todas as threads (a.0 + b.0, a.1 + b.1)
        .reduce(|| (0, 0), |a, b| (a.0 + b.0, a.1 + b.1)); 

    // ========================================================
    // TOMADA DE DECISÃO COM CERTEZA ABSOLUTA
    // ========================================================

    // 1. Array Ordenado (Nenhum degrau descendo, imune a repetidos)
    if descendo == 0 {
        return true; 
    }

    // 2. Array Invertido (Nenhum degrau subindo, imune a repetidos)
    if subindo == 0 {
        arr.reverse();
        return true;
    }

    // 3. O Dente de Serra (Sawtooth com precisão absoluta)
    // Apenas descidas muito raras (menos de 5% do array)
    if descendo > 0 && descendo < (n / 20) {
        return false; 
    }

    // 4. Caos Total
    false
}
// ==========================================
// 2. MOTOR PRINCIPAL ADAPTATIVO (ATUALIZADO)
// ==========================================

pub fn ordenar_multi_merge<T: Ord + Clone + Send + Sync>(arr: &mut [T]) {
    let n = arr.len();
    if n < 1024 {
        arr.sort_unstable(); // <- SOLUÇÃO 1
        return;
    }
    
    // 1. Intercepta ordenações perfeitas ou inversas em O(n)
    if detectar_tendencia_global(arr) { return; }

    // 2. HEURÍSTICA DE OSCILAÇÃO (Detecção Real de Caos)
    let mut e_caos_puro = false;
    if n > 120 {
        let mid = n / 2;
        let mut mudancas_direcao = 0;
        let mut subindo = arr[mid] <= arr[mid + 1];
        
        // Analisa uma janela de 100 elementos no meio do array
        for i in (mid + 1)..(mid + 100).min(n - 1) {
            let direcao_atual = arr[i] <= arr[i + 1];
            if direcao_atual != subindo {
                mudancas_direcao += 1;
                subindo = direcao_atual;
            }
        }
        
        // Se a tendência mudar mais de 15 vezes em 100 elementos, é caos puro
        if mudancas_direcao > 15 {
            e_caos_puro = true;
        }
    }

    if !e_caos_puro {
        // ROTA A: Seu Multimerge original estável (Timsort + Merge Paralelo) para dados estruturados
        let mut buffer = vec![arr[0].clone(); n];
        let num_threads = rayon::current_num_threads();
        let threshold = (n / num_threads).max(1_000_000); 
        sort_recursivo_paralelo(arr, &mut buffer, threshold);
    } else {
        // ROTA B: BlockQuicksort paralelo in-place do Rayon para triturar o caos puro
        arr.par_sort_unstable();
    }
}

// ==========================================
// 3. NÚCLEO PROCESSADOR DO MULTIMERGE (ROTA A)
// ==========================================

fn ordenar_sequencial_timsort_style<T: Ord + Clone>(arr: &mut [T], buffer: &mut [T]) {
    let n = arr.len();
    let minrun = calcular_minrun(n);

    for i in (0..n).step_by(minrun) {
        let end = (i + minrun).min(n);
        arr[i..end].sort_unstable(); // <- SOLUÇÃO 2
    }

    let mut tamanho_bloco = minrun;
    while tamanho_bloco < n {
        for esq in (0..n).step_by(tamanho_bloco * 2) {
            let meio = (esq + tamanho_bloco).min(n);
            let dir = (esq + tamanho_bloco * 2).min(n);
            if meio < dir {
                mesclar_estavel(&mut arr[esq..dir], buffer, meio - esq);
            }
        }
        tamanho_bloco *= 2;
    }
}

fn sort_recursivo_paralelo<T: Ord + Clone + Send>(arr: &mut [T], buffer: &mut [T], threshold: usize) {
    let n = arr.len();

    if n <= threshold {
        ordenar_sequencial_timsort_style(arr, buffer);
        return;
    }

    let meio = n / 2;
    let (arr_esq, arr_dir) = arr.split_at_mut(meio);
    let (buf_esq, buf_dir) = buffer.split_at_mut(meio);

    rayon::join(
        || sort_recursivo_paralelo(arr_esq, buf_esq, threshold),
        || sort_recursivo_paralelo(arr_dir, buf_dir, threshold),
    );

    mesclar_estavel(arr, buffer, meio);
}

fn mesclar_estavel<T: Ord + Clone>(arr: &mut [T], buffer: &mut [T], meio: usize) {
    let n = arr.len();
    buffer[..n].clone_from_slice(&arr[..n]);

    let mut i = 0;
    let mut j = meio;
    let mut k = 0;

    while i < meio && j < n {
        if buffer[i] <= buffer[j] {
            arr[k] = buffer[i].clone();
            i += 1;
        } else {
            arr[k] = buffer[j].clone();
            j += 1;
        }
        k += 1;
    }

    if i < meio {
        arr[k..k + (meio - i)].clone_from_slice(&buffer[i..meio]);
    } else if j < n {
        arr[k..k + (n - j)].clone_from_slice(&buffer[j..n]);
    }
}
// ==========================================
// MÓDULO JNI (INTERFACE COM O JAVA)
// ==========================================

// Estrutura estática para texto em flat bytes (6 bytes fixos)
#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord)]
#[repr(transparent)]
pub struct StringFixa(pub [u8; 6]);
unsafe impl Send for StringFixa {}

// 1. Função Numérica Otimizada
#[no_mangle]
pub extern "system" fn Java_org_multimerge_JmhSortBenchmark_multiMergeRust<'local>(
    mut env: jni::JNIEnv<'local>,
    _class: jni::objects::JClass<'local>,
    input: jni::objects::JIntArray<'local>,
) {
    // Obtém o ponteiro crítico bloqueando o Garbage Collector (Zero-Copy)
    let mut elements = unsafe {
        env.get_array_elements_critical(&input, jni::objects::ReleaseMode::CopyBack)
            .expect("Erro ao obter elementos do array Java")
    };

    let slice: &mut [i32] = unsafe {
        std::slice::from_raw_parts_mut(elements.as_mut_ptr() as *mut i32, elements.len())
    };

    // Chama o motor core
    ordenar_multi_merge(slice);
}

// 2. Função Textual Otimizada
#[no_mangle]
pub extern "system" fn Java_org_multimerge_JmhSortBenchmark_multiMergeRustStrings<'local>(
    mut env: jni::JNIEnv<'local>,
    _class: jni::objects::JClass<'local>,
    input: jni::objects::JByteArray<'local>,
) {
    let mut elements = unsafe {
        env.get_array_elements_critical(&input, jni::objects::ReleaseMode::CopyBack)
            .expect("Erro ao obter elementos do array de bytes do Java")
    };

    let slice_i8: &mut [i8] = unsafe {
        std::slice::from_raw_parts_mut(elements.as_mut_ptr() as *mut i8, elements.len())
    };

    let slice_u8: &mut [u8] = unsafe {
        std::slice::from_raw_parts_mut(slice_i8.as_mut_ptr() as *mut u8, slice_i8.len())
    };

    let mut vetor_strings: Vec<StringFixa> = slice_u8
        .chunks_exact(6)
        .map(|chunk| {
            let mut bytes = [0u8; 6];
            bytes.copy_from_slice(chunk);
            StringFixa(bytes)
        })
        .collect();

    ordenar_multi_merge(&mut vetor_strings);

    for (i, string_fixa) in vetor_strings.iter().enumerate() {
        let idx = i * 6;
        slice_u8[idx..idx + 6].copy_from_slice(&string_fixa.0);
    }
    
// Função para o teste simples do usuário (Main.java)
#[no_mangle]
pub extern "system" fn Java_org_multimerge_Main_multiMergeRust<'local>(
    mut env: jni::JNIEnv<'local>,
    _class: jni::objects::JClass<'local>,
    input: jni::objects::JIntArray<'local>,
) {
    let mut elements = unsafe {
        env.get_array_elements_critical(&input, jni::objects::ReleaseMode::CopyBack)
            .expect("Erro ao obter elementos do array")
    };

    let slice: &mut [i32] = unsafe {
        std::slice::from_raw_parts_mut(elements.as_mut_ptr() as *mut i32, elements.len())
    };

    ordenar_multi_merge(slice);
}
}