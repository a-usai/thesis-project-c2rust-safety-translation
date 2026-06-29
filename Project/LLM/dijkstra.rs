// Dijkstra — 100 % Safe Rust, production-ready
// ─────────────────────────────────────────────────────────────────────────────
// Strategia di ownership:
//   • Grafo: Vec<Vec<Edge>> — lista di adiacenza dinamica; nessuna matrice
//     20×20 fissa, nessun raw pointer, dimensione a runtime.
//   • Coda di priorità: BinaryHeap<Reverse<(u32, usize)>> — min-heap stdlib
//     O(log V); rimpiazza la coda manuale + qsort + transmute del C originale.
//   • Distanze: Vec<Option<u32>> — None = nodo non raggiunto; elimina il
//     magic number 999 / INT_MAX del codice C2Rust.
//   • Zero unsafe, zero libc, zero stato globale mutabile.
// ─────────────────────────────────────────────────────────────────────────────

use std::cmp::Reverse;
use std::collections::BinaryHeap;
use std::io::{self, Read};

// ── Tipi ──────────────────────────────────────────────────────────────────────

/// Arco pesato e diretto nel grafo.
#[derive(Debug, Clone)]
pub struct Edge {
    /// Vertice di destinazione (indice nella lista di adiacenza).
    pub to: usize,
    /// Peso non-negativo dell'arco.
    pub weight: u32,
}

// ── Costruzione del grafo ─────────────────────────────────────────────────────

/// Converte una matrice di adiacenza quadrata in una lista di adiacenza.
///
/// Un valore `0` indica l'assenza di arco; qualsiasi valore positivo è il
/// peso dell'arco corrispondente. La matrice può essere asimmetrica (grafo
/// orientato).
pub fn matrix_to_graph(matrix: &[Vec<u32>]) -> Vec<Vec<Edge>> {
    matrix
        .iter()
        .map(|row| {
            row.iter()
                .copied()                          // &u32 → u32 (Copy)
                .enumerate()
                .filter(|&(_, w)| w > 0)           // salta archi assenti
                .map(|(v, w)| Edge { to: v, weight: w })
                .collect()
        })
        .collect()
}

// ── Algoritmo di Dijkstra ─────────────────────────────────────────────────────

/// Calcola i cammini minimi dal vertice `source` a tutti gli altri vertici
/// usando l'algoritmo di Dijkstra con un min-heap.
///
/// Restituisce `Vec<Option<u32>>` dove:
/// - `Some(d)` = distanza minima da `source` al vertice `i`.
/// - `None`    = vertice `i` non raggiungibile da `source`.
///
/// Complessità: O((V + E) log V).
///
/// # Panics
/// Se `source >= graph.len()`.
pub fn dijkstra(graph: &[Vec<Edge>], source: usize) -> Vec<Option<u32>> {
    let n = graph.len();
    let mut dist: Vec<Option<u32>> = vec![None; n];
    dist[source] = Some(0);

    // BinaryHeap è un max-heap; Reverse ne inverte l'ordinamento → min-heap.
    // Ogni voce: Reverse((distanza_corrente, vertice)).
    let mut heap: BinaryHeap<Reverse<(u32, usize)>> = BinaryHeap::new();
    heap.push(Reverse((0, source)));

    while let Some(Reverse((d, u))) = heap.pop() {
        // Voce obsoleta: è già stato trovato un percorso più breve verso u.
        if dist[u].is_some_and(|best| d > best) {
            continue;
        }

        for edge in &graph[u] {
            // saturating_add previene overflow silenzioso su input anomali.
            let candidate = d.saturating_add(edge.weight);
            let improves = dist[edge.to].is_none_or(|curr| candidate < curr);
            if improves {
                dist[edge.to] = Some(candidate);
                heap.push(Reverse((candidate, edge.to)));
            }
        }
    }

    dist
}

// ── Entry point ───────────────────────────────────────────────────────────────

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("Enter the number of vertices, then the V×V adjacency matrix:");

    // Legge tutto lo stdin in una volta: compatibile con input piped e
    // reindirizzamento da file, tipico negli ambienti di test.
    let mut input = String::new();
    io::stdin().read_to_string(&mut input)?;
    let mut tokens = input.split_whitespace();

    // Numero di vertici
    let v: usize = tokens
        .next()
        .ok_or("missing vertex count")?
        .parse()?;

    if v == 0 {
        return Err("vertex count must be at least 1".into());
    }

    // Matrice di adiacenza V×V
    let mut matrix = vec![vec![0_u32; v]; v];
    for row in matrix.iter_mut() {
        for cell in row.iter_mut() {
            *cell = tokens
                .next()
                .ok_or("insufficient matrix values")?
                .parse()?;
        }
    }

    // Costruisce il grafo, esegue Dijkstra dal vertice 0
    let graph = matrix_to_graph(&matrix);
    let distances = dijkstra(&graph, 0);

    // Stampa risultati: None → "INF" (nodo non raggiungibile)
    println!("\nNode\tDist");
    for (node, dist) in distances.iter().enumerate() {
        match dist {
            Some(d) => println!("{node}\t{d}"),
            None    => println!("{node}\tINF"),
        }
    }

    Ok(())
}