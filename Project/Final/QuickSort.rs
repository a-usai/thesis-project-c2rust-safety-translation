/// Partiziona `arr` in-place con lo schema di Hoare (pivot = `arr[0]`).
///
/// Invarianti post-chiamata:
///   - Tutti gli elementi in `arr[..pi]` sono ≤ pivot.
///   - Tutti gli elementi in `arr[pi+1..]` sono > pivot.
///   - `arr[pi]` contiene il pivot nella sua posizione finale ordinata.
///
/// Restituisce `pi`: l'indice finale del pivot all'interno di `arr`.
///
/// # Edge case
/// Se `arr` ha 0 o 1 elementi la funzione restituisce `0` immediatamente,
/// senza accedere ad alcun indice — impossibile ottenere un panic.
fn partition(arr: &mut [i32]) -> usize {
    // Guardia: una slice di lunghezza ≤ 1 è già partizionata.
    if arr.len() <= 1 {
        return 0;
    }

    let len = arr.len();
    let pivot = arr[0];         // C: int p = arr[low];  (low == 0 nella slice)
    let mut i: usize = 0;       // C: int i = low;
    let mut j: usize = len - 1; // C: int j = high;  (high == len - 1)

    // C: while (i < j) { ... }
    while i < j {
        // C: while (arr[i] <= p && i <= high - 1) i++;
        //    `i <= high - 1`  ≡  `i < len - 1`
        while arr[i] <= pivot && i < len - 1 {
            i += 1;
        }

        // C: while (arr[j] > p && j >= low + 1) j--;
        //    `j >= low + 1`  ≡  `j > 0`
        //    Il controllo `j > 0` prima del decremento previene l'underflow
        //    di `usize` (comportamento definito in Rust debug, UB in C release).
        while arr[j] > pivot && j > 0 {
            j -= 1;
        }

        // C: if (i < j) swap(&arr[i], &arr[j]);
        if i < j {
            arr.swap(i, j);
        }
    }

    // C: swap(&arr[low], &arr[j]);
    // Porta il pivot nella sua posizione finale (indice j).
    arr.swap(0, j);

    j // C: return j;
}

/// Ordina `arr` in-place con l'algoritmo QuickSort (schema di Hoare).
///
/// Complessità: O(n log n) caso medio, O(n²) caso peggiore (array già ordinato
/// o con tutti elementi uguali).
///
/// La funzione è ricorsiva e completamente safe:
///   - Il caso base `len ≤ 1` evita accessi fuori limite.
///   - `split_at_mut(pi)` produce due slice mutabili non sovrapposte,
///     soddisfacendo il borrow checker senza blocchi `unsafe`.
///   - `right[1..]` salta il pivot (già in posizione), replicando
///     `quickSort(arr, pi + 1, high)` del C.
pub fn quick_sort(arr: &mut [i32]) {
    // C: if (low < high) { ... }
    // Con slice: low == 0, high == len - 1, quindi low < high  ≡  len > 1.
    if arr.len() <= 1 {
        return;
    }

    let pi = partition(arr);

    // split_at_mut(pi) restituisce:
    //   left  = arr[0..pi]   → partizione sinistra (tutti ≤ pivot)
    //   right = arr[pi..]    → pivot + partizione destra
    // right[0] == arr[pi] è il pivot già in posizione: viene saltato con right[1..].
    let (left, right) = arr.split_at_mut(pi);

    quick_sort(left);            // C: quickSort(arr, low, pi - 1);
    quick_sort(&mut right[1..]); // C: quickSort(arr, pi + 1, high);
}

fn main() {
    let mut arr = [4_i32, 2, 5, 3, 1]; // C: int arr[] = { 4, 2, 5, 3, 1 };

    quick_sort(&mut arr); // C: quickSort(arr, 0, n - 1);

    // C: for (int i = 0; i < n; i++) printf("%d ", arr[i]);
    // `print!` (senza 'ln') + spazio dopo ogni elemento replica esattamente
    // l'output del C, incluso lo spazio finale e l'assenza di '\n' terminale.
    for x in arr.iter() {
        print!("{} ", x);
    }
}