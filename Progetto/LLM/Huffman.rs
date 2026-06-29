// Strategia di ownership adottata
// ──────────────────────────────────────────────────────────────────────────────
// MinHeap è rappresentato come una struct che possiede un Vec<Box<MinHeapNode>>.
// Vec garantisce allocazione/deallocazione automatica (nessun malloc/free).
// I nodi figli dell'albero sono modellati come Option<Box<MinHeapNode>>:
//   - None  ≡  puntatore NULL originale
//   - Some(Box<T>) ≡ nodo figlio posseduto in modo esclusivo (no aliasing)
// L'heap usa indici usize al posto di puntatori raw per navigare l'array interno,
// eliminando ogni possibilità di dangling pointer o buffer overflow.
// Tutte le operazioni sono 100 % Safe Rust: nessun blocco unsafe, nessuna libc.
// ──────────────────────────────────────────────────────────────────────────────

/// Nodo di un albero binario usato dall'algoritmo di Huffman.
#[derive(Clone)]
pub struct MinHeapNode {
    pub data: u8,
    pub freq: u32,
    pub left:  Option<Box<MinHeapNode>>,
    pub right: Option<Box<MinHeapNode>>,
}

impl MinHeapNode {
    /// Crea un nuovo nodo foglia con il carattere `data` e frequenza `freq`.
    pub fn new(data: u8, freq: u32) -> Box<Self> {
        Box::new(MinHeapNode {
            data,
            freq,
            left:  None,
            right: None,
        })
    }
}

/// Min-Heap di nodi Huffman, ownership esclusiva tramite Vec<Box<MinHeapNode>>.
pub struct MinHeap {
    /// Elementi attualmente presenti nell'heap (slice logica di `array`).
    pub size:     usize,
    /// Capacità massima allocata.
    pub capacity: usize,
    /// Storage interno: gli elementi [0..size) sono la heap property.
    array: Vec<Box<MinHeapNode>>,
}

impl MinHeap {
    /// Costruisce un MinHeap vuoto con la capacità specificata.
    pub fn with_capacity(capacity: usize) -> Self {
        MinHeap {
            size: 0,
            capacity,
            array: Vec::with_capacity(capacity),
        }
    }

    // ── Heapify ──────────────────────────────────────────────────────────────

    /// Ripristina la proprietà Min-Heap a partire dall'indice `idx` verso il basso.
    ///
    /// Complessità: O(log n).
    pub fn min_heapify(&mut self, idx: usize) {
        let left  = 2 * idx + 1;
        let right = 2 * idx + 2;
        let mut smallest = idx;

        if left < self.size && self.array[left].freq < self.array[smallest].freq {
            smallest = left;
        }
        if right < self.size && self.array[right].freq < self.array[smallest].freq {
            smallest = right;
        }

        if smallest != idx {
            self.array.swap(smallest, idx);
            self.min_heapify(smallest);
        }
    }

    // ── Extract-min ──────────────────────────────────────────────────────────

    /// Estrae e restituisce il nodo con frequenza minima.
    ///
    /// Restituisce `None` se l'heap è vuoto.
    pub fn extract_min(&mut self) -> Option<Box<MinHeapNode>> {
        if self.size == 0 {
            return None;
        }

        // Scambia radice con l'ultimo elemento, poi rimuove l'ultimo.
        let last = self.size - 1;
        self.array.swap(0, last);
        self.size -= 1;
        let min_node = self.array.pop(); // rimuove l'ex-radice ora in fondo

        // Ripristina la heap property dalla radice.
        if self.size > 0 {
            self.min_heapify(0);
        }

        min_node
    }

    // ── Insert ───────────────────────────────────────────────────────────────

    /// Inserisce un nodo nell'heap, mantenendo la heap property.
    ///
    /// Complessità: O(log n).
    pub fn insert(&mut self, node: Box<MinHeapNode>) {
        assert!(self.size < self.capacity, "MinHeap: capacità esaurita");
        self.array.push(node);
        self.size += 1;

        // Bubble-up: risale fino a ristabilire la heap property.
        let mut i = self.size - 1;
        while i > 0 {
            let parent = (i - 1) / 2;
            if self.array[i].freq < self.array[parent].freq {
                self.array.swap(i, parent);
                i = parent;
            } else {
                break;
            }
        }
    }

    /// Restituisce `true` se l'heap contiene un solo elemento.
    pub fn is_size_one(&self) -> bool {
        self.size == 1
    }
}

// ── Huffman build ─────────────────────────────────────────────────────────────

/// Costruisce l'albero di Huffman a partire da vettori di dati e frequenze.
///
/// # Panics
/// Se `data` e `freq` hanno lunghezze diverse.
pub fn build_huffman_tree(data: &[u8], freq: &[u32]) -> Box<MinHeapNode> {
    assert_eq!(data.len(), freq.len(), "data e freq devono avere la stessa lunghezza");
    let n = data.len();

    let mut heap = MinHeap::with_capacity(n);
    for i in 0..n {
        heap.insert(MinHeapNode::new(data[i], freq[i]));
    }

    // Finché nell'heap ci sono almeno due nodi, combina i due minimi.
    while !heap.is_size_one() {
        let left  = heap.extract_min().expect("heap non vuoto: left");
        let right = heap.extract_min().expect("heap non vuoto: right");

        // Nodo interno: carattere sentinella '$', freq = somma dei figli.
        let combined_freq = left.freq + right.freq;
        let internal = Box::new(MinHeapNode {
            data:  b'$',
            freq:  combined_freq,
            left:  Some(left),
            right: Some(right),
        });
        heap.insert(internal);
    }

    heap.extract_min().expect("l'albero deve avere almeno un nodo")
}

// ── Stampa codici Huffman ─────────────────────────────────────────────────────

/// Stampa ricorsivamente i codici di Huffman per ogni foglia dell'albero.
pub fn print_codes(node: &MinHeapNode, code: &mut Vec<u8>) {
    if node.left.is_none() && node.right.is_none() {
        let code_str: String = code.iter().map(|b| *b as char).collect();
        println!("'{}' (freq {}): {}", node.data as char, node.freq, code_str);
        return;
    }
    if let Some(left) = &node.left {
        code.push(b'0');
        print_codes(left, code);
        code.pop();
    }
    if let Some(right) = &node.right {
        code.push(b'1');
        print_codes(right, code);
        code.pop();
    }
}

// ── Entry point ───────────────────────────────────────────────────────────────

fn main() {
    let data = [b'a', b'b', b'c', b'd', b'e', b'f'];
    let freq = [5u32, 9, 12, 13, 16, 45];

    let root = build_huffman_tree(&data, &freq);
    println!("Codici di Huffman:");
    print_codes(&root, &mut Vec::new());
}