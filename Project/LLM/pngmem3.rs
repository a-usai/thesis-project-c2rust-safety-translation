//! # PNG Memory Manager — 100% Safe Rust 2024 Edition
//!
//! Refactoring radicale del modulo `pngmem` originariamente transpilato da C2Rust.
//! Tutti i vincoli FFI, puntatori grezzi e dipendenze `libc` sono stati eliminati.
//!
//! ## Trasformazioni architetturali rispetto al codice C2Rust
//!
//! | Costrutto C / C2Rust                  | Equivalente Rust idiomatico                  |
//! |---------------------------------------|----------------------------------------------|
//! | `exit(1)` in `png_error`              | `Err(PngError::…)` propagato                 |
//! | `*mut c_void` / raw pointer           | `Vec<u8>` con ownership automatica           |
//! | `size_t` / `libc::c_int` + cast chain | `usize` + `checked_mul`                      |
//! | Cast `i32 → u32 → u64` (heap corrupt) | Eliminato — solo `usize` nativo              |
//! | `memset` prima di `free` (dead-store) | `impl Drop` + `zeroize` (F1)                 |
//! | `malloc` / `calloc` come API separata | API unificata `alloc` con zeroing (F2)       |
//! | `png_free(ptr)` manuale               | Drop automatico di `Vec<u8>`                 |
//! | `libc::malloc` / `libc::calloc`       | `Vec::try_reserve_exact` (non-panic)         |
//! | Guardia tautologica `size > u64::MAX` | Eliminata (era strutturalmente morta)        |
//! | `png_struct_def` (`snake_case`)        | `PngMem` (PascalCase RFC 430)               |
//! | `#[allow(clippy::…)]` soppressioni    | Nessuna — zero warning nativi               |

use std::fmt;

use zeroize::Zeroize;

// ── Errori ────────────────────────────────────────────────────────────────────

/// Errori del gestore di memoria PNG.
///
/// Sostituisce il meccanismo `exit(1)` di `png_error()` nella versione C originale,
/// consentendo la propagazione sicura degli errori al chiamante tramite `Result`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PngError {
    /// Allocazione fallita: memoria insufficiente.
    OutOfMemory,
    /// Argomento non valido (dimensione zero, contatore non positivo, ecc.).
    InvalidArgument(&'static str),
    /// Overflow nel calcolo `nelements × element_size`.
    SizeOverflow,
}

impl fmt::Display for PngError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            PngError::OutOfMemory => write!(f, "PNG memory: out of memory"),
            PngError::InvalidArgument(msg) => write!(f, "PNG memory: invalid argument — {msg}"),
            PngError::SizeOverflow => write!(f, "PNG memory: size overflow"),
        }
    }
}

impl std::error::Error for PngError {}

// ── Gestore di memoria PNG ────────────────────────────────────────────────────

/// Gestore di memoria PNG.
///
/// Sostituisce la `png_struct` dell'ABI C originale come contesto per le operazioni
/// di allocazione e diagnostica. La deallocazione delle risorse avviene
/// automaticamente tramite il sistema di ownership di Rust: non è necessaria
/// una funzione `png_free` esplicita.
///
/// # Sicurezza della memoria
/// All'uscita dallo scope, [`Drop`] invoca [`zeroize::Zeroize::zeroize`] sul campo
/// `label`, garantendo l'azzeramento fisico in memoria in modo resistente
/// all'ottimizzatore del compilatore (equivalente a `write_volatile` nella versione C).
#[derive(Debug)]
pub struct PngMem {
    label: String,
}

impl PngMem {
    /// Crea un nuovo contesto con l'etichetta diagnostica fornita.
    pub fn new(label: impl Into<String>) -> Self {
        PngMem { label: label.into() }
    }

    /// Emette un messaggio di warning diagnostico su stderr.
    ///
    /// Corrisponde a `png_warning()` nella versione C originale.
    /// In un'implementazione più completa, potrebbe invocare una callback utente
    /// registrata nel contesto.
    pub fn warn(&self, message: &str) {
        eprintln!("[PNG WARN] {}: {message}", self.label);
    }

    // ── Primitiva interna ─────────────────────────────────────────────────────

    /// Tenta di allocare `size` byte azzerati; restituisce `None` su OOM o size == 0.
    ///
    /// Usa [`Vec::try_reserve_exact`] per una gestione dell'OOM non-panicking,
    /// a differenza di `Vec::with_capacity` che può abortire il processo su
    /// sistemi senza overcommit della memoria.
    fn alloc_base(size: usize) -> Option<Vec<u8>> {
        if size == 0 {
            return None;
        }
        let mut buf: Vec<u8> = Vec::new();
        buf.try_reserve_exact(size).ok()?;
        buf.resize(size, 0);
        Some(buf)
    }

    // ── API pubblica ──────────────────────────────────────────────────────────

    /// Alloca `size` byte e li restituisce come buffer sempre inizializzato a zero.
    ///
    /// A differenza della coppia `malloc`/`calloc` del C originale — dove `malloc`
    /// restituiva memoria non inizializzata e `calloc` la azzerava — questa
    /// implementazione unifica le due API: [`alloc_base`][Self::alloc_base] usa
    /// `Vec::resize(..., 0)` che azzera sistematicamente ogni byte allocato.
    /// L'API è quindi più sicura e onesta: nessun buffer non inizializzato può
    /// essere restituito al chiamante per errore.
    ///
    /// Corrisponde a `png_malloc()` + `png_calloc()` nella versione C originale.
    ///
    /// # Errors
    /// - [`PngError::InvalidArgument`] se `size == 0`.
    /// - [`PngError::OutOfMemory`] se l'allocazione fallisce.
    pub fn alloc(&self, size: usize) -> Result<Vec<u8>, PngError> {
        if size == 0 {
            return Err(PngError::InvalidArgument("size must be > 0"));
        }
        Self::alloc_base(size).ok_or(PngError::OutOfMemory)
    }

    /// Tenta di allocare `size` byte; emette un warning su stderr in caso di OOM.
    ///
    /// Corrisponde a `png_malloc_warn()` nella versione C originale.
    /// Restituisce `None` in caso di fallimento, senza terminare il processo.
    /// Non emette warning per `size == 0` (errore di utilizzo del chiamante,
    /// non una condizione di OOM).
    pub fn alloc_warn(&self, size: usize) -> Option<Vec<u8>> {
        let result = Self::alloc_base(size);
        if result.is_none() && size > 0 {
            self.warn("out of memory");
        }
        result
    }

    /// Alloca un buffer per `nelements` elementi di `element_size` byte ciascuno.
    ///
    /// Corrisponde a `png_malloc_array()` nella versione C originale.
    /// La dimensione totale è calcolata con [`usize::checked_mul`] per prevenire
    /// overflow silenzioso — difetto critico (F4/F6) della versione C2Rust, dove
    /// la catena di cast `libc::c_int → libc::c_uint → libc::c_ulong` su valori
    /// negativi produceva dimensioni enormi (~4 GiB), causando heap corruption.
    ///
    /// # Errors
    /// - [`PngError::InvalidArgument`] se `nelements == 0` o `element_size == 0`.
    /// - [`PngError::SizeOverflow`] se `nelements × element_size > usize::MAX`.
    /// - [`PngError::OutOfMemory`] se l'allocazione fallisce.
    pub fn alloc_array(
        &self,
        nelements: usize,
        element_size: usize,
    ) -> Result<Vec<u8>, PngError> {
        if nelements == 0 {
            return Err(PngError::InvalidArgument("nelements must be > 0"));
        }
        if element_size == 0 {
            return Err(PngError::InvalidArgument("element_size must be > 0"));
        }
        let total = nelements
            .checked_mul(element_size)
            .ok_or(PngError::SizeOverflow)?;
        Self::alloc_base(total).ok_or(PngError::OutOfMemory)
    }

    /// Estende il buffer `old` aggiungendo `add_elements` elementi di `element_size` byte.
    ///
    /// Corrisponde a `png_realloc_array()` nella versione C originale.
    /// I nuovi byte sono azzerati; il contenuto esistente è conservato in-place.
    /// Consuma `old` per ownership, restituendo il buffer esteso senza copie
    /// superflue grazie alla gestione dell'allocatore interno di `Vec`.
    ///
    /// # Errors
    /// - [`PngError::InvalidArgument`] se `add_elements == 0` o `element_size == 0`.
    /// - [`PngError::SizeOverflow`] se il calcolo della nuova dimensione va in overflow.
    /// - [`PngError::OutOfMemory`] se la riallocazione fallisce.
    pub fn realloc_array(
        &self,
        mut old: Vec<u8>,
        add_elements: usize,
        element_size: usize,
    ) -> Result<Vec<u8>, PngError> {
        if add_elements == 0 {
            return Err(PngError::InvalidArgument("add_elements must be > 0"));
        }
        if element_size == 0 {
            return Err(PngError::InvalidArgument("element_size must be > 0"));
        }
        let add_bytes = add_elements
            .checked_mul(element_size)
            .ok_or(PngError::SizeOverflow)?;
        let new_size = old
            .len()
            .checked_add(add_bytes)
            .ok_or(PngError::SizeOverflow)?;
        // `new_size >= old.len()` è garantito da checked_add con add_bytes > 0.
        let extra = new_size - old.len();
        old.try_reserve_exact(extra)
            .map_err(|_| PngError::OutOfMemory)?;
        old.resize(new_size, 0);
        Ok(old)
    }
}

impl Default for PngMem {
    fn default() -> Self {
        PngMem::new("png")
    }
}

impl Drop for PngMem {
    /// Azzera il campo `label` prima della deallocazione.
    ///
    /// Corrisponde a `png_destroy_png_struct()` con `write_volatile` nella
    /// versione C originale. L'uso di [`Zeroize::zeroize`] garantisce che
    /// l'ottimizzatore non elimini la scrittura come dead-store, prevenendo
    /// data leakage residui nell'heap dopo il drop del contesto.
    fn drop(&mut self) {
        self.label.zeroize();
    }
}

// ── Entry point ───────────────────────────────────────────────────────────────

fn main() {
    let mem = PngMem::new("demo");

    // ── alloc: zeroing garantito ──────────────────────────────────────────────
    match mem.alloc(256) {
        Ok(buf) => println!("alloc(256)         → {} byte (sempre zero) ✓", buf.len()),
        Err(e)  => eprintln!("alloc error: {e}"),
    }

    // ── alloc a 64 byte: verifica azzeramento (unifica calloc + malloc) ────────
    let buf = mem.alloc(64).expect("alloc(64) failed");
    assert!(buf.iter().all(|&b| b == 0), "alloc deve sempre restituire buffer azzerato");
    println!("alloc(64)          → {} byte, tutti zero ✓", buf.len());

    // ── alloc_array: 10 elementi × 4 byte ────────────────────────────────────
    let arr = mem.alloc_array(10, 4).expect("alloc_array(10,4) failed");
    println!("alloc_array(10, 4) → {} byte", arr.len());

    // ── realloc_array: estendi di 5 elementi ──────────────────────────────────
    let extended = mem.realloc_array(arr, 5, 4).expect("realloc_array failed");
    assert_eq!(extended.len(), 60, "10+5 elementi × 4 byte = 60 byte");
    println!("realloc_array(+5)  → {} byte totali ✓", extended.len());

    // ── alloc_warn: size=0 → None senza warning ───────────────────────────────
    let none_result = mem.alloc_warn(0);
    assert!(none_result.is_none());
    println!("alloc_warn(0)      → None, nessun warning per size=0 ✓");

    // ── SizeOverflow: checked_mul intercetta l'overflow ───────────────────────
    let overflow = mem.alloc_array(usize::MAX, 2);
    assert_eq!(overflow, Err(PngError::SizeOverflow));
    println!("alloc_array(MAX,2) → SizeOverflow rilevato ✓");

    // ── InvalidArgument ───────────────────────────────────────────────────────
    let bad = mem.alloc(0);
    assert_eq!(bad, Err(PngError::InvalidArgument("size must be > 0")));
    println!("alloc(0)           → InvalidArgument ✓");

    // ── Drop automatico con zeroize ───────────────────────────────────────────
    // `mem` viene droppato qui: `impl Drop` invoca `self.label.zeroize()`
    // garantendo l'azzeramento fisico del campo prima della deallocazione.
    // Non è necessaria alcuna chiamata esplicita a `destroy()`.
    drop(mem);
    println!("drop(mem)          → zeroize via impl Drop ✓");
}