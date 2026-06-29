// QuickSort — Safe Rust, production-ready
// ─────────────────────────────────────────────────────────────────────────────
// Strategia di ownership:
//   • L'array è passato come slice mutabile `&mut [i32]`, eliminando ogni
//     puntatore raw e aritmetica unsafe del C.
//   • Il partizionamento usa `arr.swap(i, j)` — API stdlib sicura, zero unsafe.
//   • La ricorsione usa `split_at_mut` per produrre due slice mutabili
//     non sovrapposte, rispettando il borrow checker senza alcun blocco unsafe.
//   • I casi base (len ≤ 1) sono gestiti prima di qualsiasi accesso
//     agli elementi, rendendo impossibili panic per indici fuori limite.
// ─────────────────────────────────────────────────────────────────────────────

/// Partiziona `arr` con schema di Hoare (pivot = `arr[0]`).
///
/// Al termine della funzione:
/// - Tutti gli elementi in `arr[..pi]` sono ≤ pivot.
/// - Tutti gli elementi in `arr[pi+1..]` sono > pivot.
/// - `arr[pi]` è il pivot nella sua posizione finale ordinata.
///
/// Restituisce l'indice finale del pivot all'interno di `arr`.
fn partition(arr: &mut [i32]) -> usize {
    // Guardia: slice di 0 o 1 elementi è già partizionata.
    if arr.len() <= 1 {
        return 0;
    }

    let len = arr.len();
    let pivot = arr[0];
    let mut i: usize = 0;
    let mut j: usize = len - 1;

    loop {
        // Avanza i verso destra finché arr[i] ≤ pivot (fino a len-2 al massimo).
        while i < len - 1 && arr[i] <= pivot {
            i += 1;
        }
        // Arretra j verso sinistra finché arr[j] > pivot (fino a 1 al minimo).
        // Il controllo `j > 0` prima del decremento previene l'underflow di usize.
        while j > 0 && arr[j] > pivot {
            j -= 1;
        }
        if i >= j {
            break;
        }
        arr.swap(i, j);
    }

    // Porta il pivot nella sua posizione finale (indice j).
    arr.swap(0, j);
    j
}

/// Ordina `arr` in-place con l'algoritmo QuickSort (schema di Hoare).
///
/// Complessità: O(n log n) caso medio, O(n²) caso peggiore.
///
/// La funzione è ricorsiva ma sicura:
/// - Il caso base `len ≤ 1` evita accessi fuori limite e stack overflow
///   su input degeneri di piccole dimensioni.
/// - `split_at_mut` produce due slice mutabili non sovrapposte, soddisfacendo
///   il borrow checker senza ricorrere a blocchi `unsafe`.
pub fn quick_sort(arr: &mut [i32]) {
    if arr.len() <= 1 {
        return;
    }

    let pi = partition(arr);

    // split_at_mut(pi) → (arr[0..pi], arr[pi..])
    // right[0] = arr[pi] è il pivot già in posizione: viene saltato con right[1..].
    let (left, right) = arr.split_at_mut(pi);
    quick_sort(left);            // ordina la partizione sinistra [0..pi)
    quick_sort(&mut right[1..]); // ordina la partizione destra (pi..len)
}

fn main() {
    let mut arr = [4_i32, 2, 5, 3, 1];
    quick_sort(&mut arr);

    let parts: Vec<String> = arr.iter().map(|x| x.to_string()).collect();
    println!("{}", parts.join(" "));
}