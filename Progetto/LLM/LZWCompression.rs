//! LZW Compression — Safe Rust, production-ready
//! ─────────────────────────────────────────────────────────────────────────────
//! Strategia di ownership:
//!   • Dizionario: `HashMap<Vec<u8>, u16>` — ogni sequenza è posseduta dalla
//!     mappa; nessun raw pointer, malloc o C-string.
//!   • Sequenze intermedie: `Vec<u8>` stack-owned, mosse nel dizionario senza
//!     allocazioni non necessarie.
//!   • Nessun blocco `unsafe`, nessuna dipendenza `libc`, zero `panic!`/`assert!`.
//!   • Errori di runtime propagati tramite `Result<T, LzwError>`.
//!   • La ricerca nel dizionario è O(1) ammortizzato (HashMap) invece che
//!     O(n) lineare come nell'originale C2Rust.
//! ─────────────────────────────────────────────────────────────────────────────

use std::collections::HashMap;
use std::fmt;

// ── Error type ────────────────────────────────────────────────────────────────

/// Errori possibili durante la compressione LZW.
#[derive(Debug, PartialEq, Eq)]
pub enum LzwError {
    /// Il dizionario ha raggiunto la capacità massima (4 096 voci, LZW 12-bit).
    DictionaryFull,
    /// L'input fornito è vuoto: non c'è nulla da comprimere.
    EmptyInput,
    /// Stato interno inconsistente: una sequenza attesa non è nel dizionario.
    SequenceNotFound(Vec<u8>),
}

impl fmt::Display for LzwError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::DictionaryFull => write!(f, "LZW: dizionario pieno (4 096 voci)"),
            Self::EmptyInput => write!(f, "LZW: input vuoto"),
            Self::SequenceNotFound(seq) => {
                write!(f, "LZW: sequenza non trovata nel dizionario: {seq:?}")
            }
        }
    }
}

impl std::error::Error for LzwError {}

// ── Constants ─────────────────────────────────────────────────────────────────

/// Capacità massima del dizionario LZW a 12 bit (2¹² = 4 096 voci).
const MAX_DICT_SIZE: usize = 4096;

// ── Dictionary helpers ────────────────────────────────────────────────────────

/// Inizializza il dizionario con le 256 entry single-byte standard LZW.
///
/// Ogni singolo byte `b` mappa alla sequenza `[b]` con codice `u16::from(b)`.
fn init_dictionary() -> HashMap<Vec<u8>, u16> {
    (0_u8..=255_u8)
        .map(|byte| (vec![byte], u16::from(byte)))
        .collect()
}

/// Aggiunge `sequence` al dizionario assegnandole il prossimo codice disponibile.
///
/// # Errors
/// Restituisce [`LzwError::DictionaryFull`] se la dimensione ha raggiunto
/// `MAX_DICT_SIZE` (4 096 voci).
fn add_entry(dict: &mut HashMap<Vec<u8>, u16>, sequence: Vec<u8>) -> Result<u16, LzwError> {
    if dict.len() >= MAX_DICT_SIZE {
        return Err(LzwError::DictionaryFull);
    }
    // dict.len() < MAX_DICT_SIZE = 4 096 ≤ u16::MAX = 65 535:
    // la conversione è infallibile; map_err gestisce l'edge-case teorico.
    let next_code =
        u16::try_from(dict.len()).map_err(|_| LzwError::DictionaryFull)?;
    dict.entry(sequence).or_insert(next_code);
    Ok(next_code)
}

// ── Compression ───────────────────────────────────────────────────────────────

/// Comprime `input` tramite l'algoritmo LZW e restituisce i codici prodotti.
///
/// La funzione è completamente iterativa (nessuna ricorsione).
/// La complessità algoritmica è O(n) sul numero di byte in input,
/// con lookup O(1) ammortizzato grazie a `HashMap`.
///
/// # Errors
/// - [`LzwError::EmptyInput`] se `input` è vuoto.
/// - [`LzwError::DictionaryFull`] se il dizionario raggiunge 4 096 voci
///   prima che la compressione sia completata.
/// - [`LzwError::SequenceNotFound`] in caso di stato interno inconsistente.
pub fn compress(input: &[u8]) -> Result<Vec<u16>, LzwError> {
    if input.is_empty() {
        return Err(LzwError::EmptyInput);
    }

    let mut dict = init_dictionary();
    let mut output: Vec<u16> = Vec::new();

    // `current` accumula la sequenza corrente (longest match finora trovato).
    let mut current = vec![input[0]];

    for &byte in &input[1..] {
        // Estende la sequenza corrente con il byte successivo.
        let candidate: Vec<u8> = current
            .iter()
            .copied()
            .chain(std::iter::once(byte))
            .collect();

        if dict.contains_key(&candidate) {
            // La sequenza estesa è nel dizionario: continua ad accumulare.
            current = candidate;
        } else {
            // Emette il codice di `current` (sequenza più lunga trovata).
            let code = dict
                .get(&current)
                .copied()
                .ok_or_else(|| LzwError::SequenceNotFound(current.clone()))?;
            output.push(code);

            // Aggiunge `candidate` al dizionario; propaga l'errore se pieno.
            add_entry(&mut dict, candidate)?;

            // Ricomincia da `byte` come nuova sequenza corrente.
            current = vec![byte];
        }
    }

    // Emette il codice dell'ultima sequenza rimasta nel buffer.
    let code = dict
        .get(&current)
        .copied()
        .ok_or_else(|| LzwError::SequenceNotFound(current.clone()))?;
    output.push(code);

    Ok(output)
}

// ── Entry point ───────────────────────────────────────────────────────────────

fn main() {
    let input = b"TOBEORNOTTOBEORTOBEORNOT";

    match compress(input) {
        Ok(codes) => {
            let parts: Vec<String> = codes.iter().map(|c| c.to_string()).collect();
            println!("{}", parts.join(" "));
        }
        Err(e) => eprintln!("Errore di compressione: {e}"),
    }
}