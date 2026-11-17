# java-multimerge-dll — MultiMerge chamado do Java por JNI

Biblioteca nativa em Rust (`cdylib`) mais um benchmark JMH em Java. Clone de
`https://github.com/fbcouto/java-multimerge-dll`, feito em 2026-11-13 para
receber o motor novo.

A pasta `D:\_multimerge\SUBMISSOES\Rust_Generico_Java_dll` é uma versão mais
antiga do mesmo projeto (sem git, exports `Java_BenchmarkSort_*`). Ela recebeu
o mesmo porte, mas **este clone é a versão que vai para o GitHub**.

## Combinados com o autor (Fernando; fala português)

- Nomes: **Multimerge = instável e padrão; Multimerge_Stable = estável.**
  Decisão dele, não se rediscute.
- **Nada de commit nem de push sem ele pedir.** Ele testa antes.
- O motor (`src/multimerge.rs`) não se edita aqui; altera-se no projeto-base e
  copia-se de novo.
- Número sem método não vale: rodar o benchmark duas vezes. Entre rodadas
  discordantes vale a de média menor e intervalo mais estreito, porque
  interferência só soma tempo. O "Method" do README explica isso.
- O que for decidido ou descoberto entra neste arquivo; ele não reexplica.

## Versão

**2.0.0** (tag `v2.0.0`, 2026-11-17): motor substituído pelo de
`adaptive-parallel-multimerge-sort`, método nativo `multiMergeRustStable` novo,
`Main.java` com autoteste, tabela do README refeita em 1M/10M/30M. A versão
está em `Cargo.toml` e `pom.xml`; mudar nos dois.

## Armadilhas

- **`mvn clean` apagava a DLL.** Cargo e Maven usam a mesma `target/`; a receita
  "cargo build, mvn clean package, java" falhava com `no rust_multimerge in
  java.library.path` porque a DLL já não existia. Corrigido no `pom.xml`: o
  `maven-clean-plugin` exclui `release/`, `debug/`, `.rustc_info.json` e
  `CACHEDIR.TAG`. Esse erro do Java também aparece quando a DLL existe mas não
  carrega (dependência faltando): o JDK não distingue.
- Nesta máquina o toolchain padrão do `cargo` é `nightly-x86_64-pc-windows-msvc`.
  O projeto foi compilado e testado com `+stable-x86_64-pc-windows-gnu`.
- Se `git pull`/`push`/`clone` falhar com erro de certificado:
  `git -c http.sslBackend=schannel ...`. Não desligar a verificação.
- No Bash da ferramenta, heredoc com aspas simples ou barras invertidas estraga
  o conteúdo: grave arquivos com a ferramenta de escrita.

## O que mudou

O `lib.rs` tinha um motor próprio (Timsort + merge paralelo, escudo de ponto
único). Foi substituído por uma cópia do motor de
`adaptive-parallel-multimerge-sort`, que tem dois sorts:

| função nativa (Java) | motor |
| :--- | :--- |
| `multiMergeRust(int[])` | **Multimerge**, não estável — o padrão |
| `multiMergeRustStrings(byte[])` | idem, texto de 6 bytes fixos |
| `multiMergeRustStable(int[])` | **Multimerge_Stable** — novo |
| `multiMergeRustStringsStable(byte[])` | idem — novo |

As quatro existem para `org.multimerge.JmhSortBenchmark`. O `lib.rs` também
exporta `multiMergeRust` e `multiMergeRustStable` para uma classe
`org.multimerge.Main`, que o README cita mas **não está no repositório** (ver o
recado acima).

Com `int[]` e texto de largura fixa, elementos iguais são indistinguíveis: os
dois motores devolvem o mesmo array e a diferença é só de tempo. O benchmark
ganhou o competidor `rustMultiMergeStableSort` para medir essa diferença.

A decisão de nomes é do autor: **Multimerge = instável e padrão; o estável é o
Multimerge_Stable**, de aplicação mais restrita.

## O motor (`src/multimerge.rs`)

Cópia sem os testes. **Não editar aqui.** A fonte da verdade, os testes e as
medições estão em `D:\_multimerge\SUBMISSOES\adaptive-parallel-multimerge-sort`
(ler o `CLAUDE.md` de lá). Para atualizar a cópia:

```bash
cd D:/_multimerge/SUBMISSOES/adaptive-parallel-multimerge-sort
sh scripts/copia_do_motor.sh ../java-multimerge-dll/src/multimerge.rs
```

## Compilar e testar

```bash
cargo +stable-x86_64-pc-windows-gnu test --release    # 2 testes
cargo +stable-x86_64-pc-windows-gnu build --release   # target/release/rust_multimerge.dll
mvn -q compile                                        # lado Java
```

O `git clone` nesta máquina precisa de
`git -c http.sslBackend=schannel clone ...` (o bundle de CA do git falha).

## Validado

- Os dois testes do `lib.rs` — as quatro distribuições do benchmark (RANDOM,
  SORTED, REVERSE, SAWTOOTH) em 1M de `i32` e o texto de largura fixa, nos dois
  motores, contra o `sort` da std.
- `Main.java` por JNI: saída dos dois motores comparada elemento a elemento
  (`Arrays.equals`) com `Arrays.sort` **e** `Arrays.parallelSort`, nas 4
  distribuições, em 5, 4096, 1M, 10M e 30M (os dois últimos com
  `java -Xmx3g ... Main 10000000 30000000`). Tudo igual. Ele perguntou
  explicitamente se os arrays ordenados eram iguais nos 4 casos: a resposta
  é sim, nos 5 tamanhos.
- JMH completo em 1M, 10M e 30M, duas rodadas cada, sem falha. Números no
  README e em `benchmark-results/`.
- `mvn clean package` seguido de `java ... org.multimerge.Main` funciona (a DLL
  sobrevive ao clean).

## Como medir de novo

```powershell
cargo +stable-x86_64-pc-windows-gnu build --release
mvn clean package
java --enable-native-access=ALL-UNNAMED "-Djava.library.path=target\release" -jar target\benchmarks.jar -rf csv -rff run1.csv
java --enable-native-access=ALL-UNNAMED "-Djava.library.path=target\release" -jar target\benchmarks.jar -p size=30000000 -rf csv -rff run30m.csv
```

Uns 10 min a rodada de 1M/10M e 8 min a de 30M. Rodar duas vezes; o JMH grava
decimais com vírgula (locale pt-BR) — ao ler o CSV, trocar `,` por `.`. Com o
Git Bash, passar o `java.library.path` absoluto (`cygpath -w`) evita dúvida
sobre o caminho relativo.
