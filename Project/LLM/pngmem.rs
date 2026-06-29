unsafe extern "C" {
    fn free(_: *mut libc::c_void);
    fn malloc(_: libc::c_ulong) -> *mut libc::c_void;
    fn exit(_: libc::c_int) -> !;
    fn memcpy(
        _: *mut libc::c_void,
        _: *const libc::c_void,
        _: libc::c_ulong,
    ) -> *mut libc::c_void;
    fn memset(
        _: *mut libc::c_void,
        _: libc::c_int,
        _: libc::c_ulong,
    ) -> *mut libc::c_void;
}

#[allow(non_camel_case_types)]
pub type size_t = libc::c_ulong;

#[allow(non_camel_case_types)]
pub type png_alloc_size_t = size_t;

/// Layout interno della struttura PNG di libpng (rappresentazione C-ABI compatibile).
#[derive(Copy, Clone)]
#[repr(C)]
pub struct PngStructDef {
    pub mem_ptr: *mut libc::c_void,
}

/// Alias C-compatibile che preserva il simbolo originale `png_struct` per il linkage FFI.
///
/// `non_camel_case_types` è soppresso localmente perché l'identificatore deve corrispondere
/// all'ABI C di libpng: cambiare il nome romperebbe la compatibilità binaria.
#[allow(non_camel_case_types)]
pub type png_struct = PngStructDef;

// Rust 2024 richiede la sintassi #[unsafe(no_mangle)] per l'esportazione dei simboli

/// Termina il processo in modo incondizionato su un errore fatale di libpng.
///
/// # Safety
/// - `_png_ptr` deve essere null oppure un puntatore valido e non-dangling a `png_struct`.
/// - `_error_message` deve essere null oppure una stringa C valida e null-terminated.
/// - **Questa funzione non ritorna mai.** Chiama `exit(1)` immediatamente.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn png_error(
    _png_ptr: *const png_struct,
    _error_message: *const libc::c_char,
) {
    // SAFETY: exit() è sempre sicuro da chiamare; termina il processo immediatamente.
    unsafe { exit(1) }
}

/// Stub di warning non operativo. In libpng invocherebbe una callback dell'utente.
///
/// # Safety
/// - `_png_ptr` deve essere null oppure un puntatore valido e non-dangling a `png_struct`.
/// - `_warning_message` deve essere null oppure una stringa C valida e null-terminated.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn png_warning(
    _png_ptr: *const png_struct,
    _warning_message: *const libc::c_char,
) {
}

/// Azzera in modo sicuro e libera la `png_struct` puntata da `png_ptr`.
///
/// # Safety
/// - `png_ptr` deve essere null oppure un puntatore valido e non-dangling a una
///   `png_struct` allocata con `malloc` (o equivalente) e non ancora liberata.
/// - Dopo questa chiamata `png_ptr` diventa un dangling pointer: il chiamante
///   non deve più usarlo.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn png_destroy_png_struct(png_ptr: *mut png_struct) {
    if !png_ptr.is_null() {
        // SAFETY: png_ptr è non-null e valido per precondizione.
        //
        // F2 fix: write_volatile impedisce al compilatore di eliminare l'azzeramento
        // come dead-store prima della free() successiva, proteggendo i dati sensibili
        // residui nell'heap. L'equivalente memset() può essere rimosso dall'ottimizzatore.
        unsafe {
            ::std::ptr::write_volatile(png_ptr, ::core::mem::zeroed::<png_struct>());
            free(png_ptr as *mut libc::c_void);
        }
    }
}

/// Primitiva di allocazione di basso livello. Restituisce `null_mut()` su fallimento.
///
/// # Safety
/// - `_png_ptr` deve essere null oppure un puntatore valido e non-dangling a `png_struct`.
/// - Il puntatore restituito, se non-null, deve essere liberato con `png_free`.
///
/// **Nota (audit F3):** il controllo `size > u64::MAX` è strutturalmente tautologico
/// e non può mai essere vero; è conservato per compatibilità API.
// clippy::absurd_extreme_comparisons è soppresso localmente perché la guardia è
// un residuo intenzionale della transpilazione (difetto noto F3 dell'audit).
#[allow(clippy::absurd_extreme_comparisons)]
#[unsafe(no_mangle)]
pub unsafe extern "C" fn png_malloc_base(
    _png_ptr: *const png_struct,
    size: png_alloc_size_t,
) -> *mut libc::c_void {
    if size > 18446744073709551615 as libc::c_ulong {
        return std::ptr::null_mut();
    }
    // SAFETY: size è una richiesta di allocazione valida; malloc gestisce la chiamata OS.
    unsafe { malloc(size) }
}

/// Alloca `size` byte azzerati. Restituisce `null_mut()` su fallimento di allocazione.
///
/// # Safety
/// - `png_ptr` deve essere null oppure un puntatore valido e non-dangling a `png_struct`.
/// - Il puntatore restituito, se non-null, deve essere liberato con `png_free`.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn png_calloc(
    png_ptr: *const png_struct,
    size: png_alloc_size_t,
) -> *mut libc::c_void {
    // SAFETY: le precondizioni di png_malloc_base sono soddisfatte dalle precondizioni
    // del chiamante.
    let ret = unsafe { png_malloc_base(png_ptr, size) };
    if !ret.is_null() {
        // SAFETY: ret è un blocco di `size` byte valido e scrivibile appena allocato.
        unsafe { memset(ret, 0, size) };
    }
    ret
}

/// Helper interno di allocazione array con controllo overflow.
///
/// # Safety
/// - `png_ptr` deve essere null oppure un puntatore valido e non-dangling a `png_struct`.
/// - `nelements` deve essere positivo (i chiamanti devono validarlo prima di questa chiamata).
/// - `element_size` deve essere non-zero.
/// - Il puntatore restituito, se non-null, deve essere liberato con `png_free`.
unsafe extern "C" fn png_malloc_array_checked(
    png_ptr: *const png_struct,
    nelements: libc::c_int,
    element_size: size_t,
) -> *mut libc::c_void {
    // F6 fix: guardia esplicita contro valori negativi o zero prima del cast unsigned.
    // Un valore negativo avrebbe prodotto silenziosamente un u64 enormi, causando
    // heap corruption. Il cast i32 → u64 è sicuro solo se nelements > 0.
    if nelements <= 0 {
        return std::ptr::null_mut();
    }
    let req: png_alloc_size_t = nelements as png_alloc_size_t; // sicuro: nelements > 0
    if req <= (18446744073709551615 as libc::c_ulong).wrapping_div(element_size) {
        // SAFETY: req * element_size non va in overflow (verificato sopra).
        return unsafe { png_malloc_base(png_ptr, req.wrapping_mul(element_size)) };
    }
    std::ptr::null_mut()
}

/// Alloca un array di `nelements` elementi di `element_size` byte ciascuno.
/// Chiama `png_error` (terminazione del processo) se gli argomenti non sono validi.
///
/// # Safety
/// - `png_ptr` deve essere un puntatore valido e non-dangling a `png_struct`.
/// - `nelements` deve essere positivo.
/// - `element_size` deve essere non-zero.
/// - Il puntatore restituito, se non-null, deve essere liberato con `png_free`.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn png_malloc_array(
    png_ptr: *const png_struct,
    nelements: libc::c_int,
    element_size: size_t,
) -> *mut libc::c_void {
    if nelements <= 0 || element_size == 0 {
        // SAFETY: png_ptr è valido per precondizione; il literal di stringa è valido.
        unsafe {
            png_error(
                png_ptr,
                b"internal error: array alloc\0" as *const u8 as *const libc::c_char,
            );
        }
        // F5 fix: png_error chiama exit(1) e non ritorna mai; questo marcatore garantisce
        // al compilatore che il flusso di controllo non prosegue oltre questo punto,
        // evitando possibili UB da valori non inizializzati post-errore.
        unreachable!()
    }
    // SAFETY: nelements > 0 ed element_size > 0 sono stati validati sopra.
    unsafe { png_malloc_array_checked(png_ptr, nelements, element_size) }
}

/// Rialloca un array, copiando il contenuto esistente e azzerando la nuova regione.
///
/// # Safety
/// - `png_ptr` deve essere un puntatore valido e non-dangling a `png_struct`.
/// - `old_array`, se non-null, deve puntare a un'allocazione valida e leggibile di almeno
///   `old_elements * element_size` byte.
/// - `add_elements` deve essere positivo; `element_size` deve essere non-zero.
/// - `old_elements` deve essere non-negativo.
/// - Il puntatore restituito, se non-null, deve essere liberato con `png_free`.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn png_realloc_array(
    png_ptr: *const png_struct,
    old_array: *const libc::c_void,
    old_elements: libc::c_int,
    add_elements: libc::c_int,
    element_size: size_t,
) -> *mut libc::c_void {
    if add_elements <= 0
        || element_size == 0
        || old_elements < 0
        || (old_array.is_null() && old_elements > 0)
    {
        // SAFETY: png_ptr è valido per precondizione; il literal di stringa è valido.
        unsafe {
            png_error(
                png_ptr,
                b"internal error: array realloc\0" as *const u8 as *const libc::c_char,
            );
        }
        // F5 fix: png_error chiama exit(1) e non ritorna mai.
        unreachable!()
    }
    if add_elements <= 2147483647 as libc::c_int - old_elements {
        // SAFETY: old_elements + add_elements non va in overflow i32 (verificato sopra).
        let new_array = unsafe {
            png_malloc_array_checked(png_ptr, old_elements + add_elements, element_size)
        };
        if !new_array.is_null() {
            // F4 fix: old_elements >= 0 è validato dalla guardia sopra.
            // Il cast diretto i32 → c_ulong è sicuro e sostituisce la catena doppia
            // `as libc::c_uint as libc::c_ulong` che produceva valori enormi per input
            // negativi (i32 wrapping a u32 = ~4 GiB), causando heap corruption silenzioso.
            let old_bytes =
                element_size.wrapping_mul(old_elements as libc::c_ulong);
            if old_elements > 0 {
                // SAFETY: new_array è un'allocazione valida; old_array è valida per
                // old_bytes byte; le regioni non si sovrappongono (nuova allocazione).
                unsafe { memcpy(new_array, old_array, old_bytes) };
            }
            // F4 fix: add_elements > 0 è validato; cast diretto i32 → c_ulong sicuro.
            let add_bytes =
                element_size.wrapping_mul(add_elements as libc::c_ulong);
            // SAFETY: new_array + old_bytes è all'interno dell'allocazione di
            // (old_elements + add_elements) * element_size byte.
            unsafe {
                memset(
                    (new_array as *mut libc::c_char).add(old_bytes as usize)
                        as *mut libc::c_void,
                    0,
                    add_bytes,
                );
            }
            return new_array;
        }
    }
    std::ptr::null_mut()
}

/// Alloca `size` byte; chiama `png_error` (termina il processo) in caso di OOM.
///
/// # Safety
/// - `png_ptr` deve essere un puntatore valido, non-null e non-dangling a `png_struct`.
/// - Il puntatore restituito deve essere liberato con `png_free`.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn png_malloc(
    png_ptr: *const png_struct,
    size: png_alloc_size_t,
) -> *mut libc::c_void {
    if png_ptr.is_null() {
        return std::ptr::null_mut();
    }
    // SAFETY: png_ptr è non-null e valido per la verifica precedente.
    let ret = unsafe { png_malloc_base(png_ptr, size) };
    if ret.is_null() {
        // SAFETY: png_ptr è valido; il literal di stringa è valido.
        unsafe {
            png_error(png_ptr, b"Out of memory\0" as *const u8 as *const libc::c_char);
        }
        // F5 fix: png_error chiama exit(1) e non ritorna mai; senza questo marcatore
        // il compilatore potrebbe trattare `ret` (null) come valore di ritorno
        // valido post-errore, con rischio di dereferenziazione nel chiamante.
        unreachable!()
    }
    ret
}

/// Tenta di allocare `size` byte; emette un warning su OOM invece di terminare.
///
/// # Safety
/// - `png_ptr` deve essere null oppure un puntatore valido e non-dangling a `png_struct`.
/// - Il puntatore restituito, se non-null, deve essere liberato con `png_free`.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn png_malloc_warn(
    png_ptr: *const png_struct,
    size: png_alloc_size_t,
) -> *mut libc::c_void {
    if !png_ptr.is_null() {
        // SAFETY: png_ptr è non-null e valido per la verifica precedente.
        let ret = unsafe { png_malloc_base(png_ptr, size) };
        if !ret.is_null() {
            return ret;
        }
        // SAFETY: png_ptr è valido; il literal di stringa è valido.
        unsafe {
            png_warning(
                png_ptr,
                b"Out of memory\0" as *const u8 as *const libc::c_char,
            );
        }
    }
    std::ptr::null_mut()
}

/// Libera memoria precedentemente allocata con `png_malloc`, `png_calloc` o `png_malloc_warn`.
///
/// # Safety
/// - `png_ptr` deve essere null oppure un puntatore valido e non-dangling a `png_struct`.
/// - `ptr` deve essere null oppure un puntatore ottenuto da `png_malloc` / `png_calloc` /
///   `png_malloc_warn` e non ancora liberato.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn png_free(
    png_ptr: *const png_struct,
    ptr: *mut libc::c_void,
) {
    if png_ptr.is_null() || ptr.is_null() {
        return;
    }
    // SAFETY: ptr è un'allocazione viva e valida da malloc per precondizione.
    unsafe { free(ptr) }
}

fn main() {}