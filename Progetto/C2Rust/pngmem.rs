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

pub type size_t = libc::c_ulong;
pub type png_alloc_size_t = size_t;

#[derive(Copy, Clone)]
#[repr(C)]
pub struct png_struct_def {
    pub mem_ptr: *mut libc::c_void,
}
pub type png_struct = png_struct_def;

// Rust 2024 richiede la sintassi #[unsafe(no_mangle)] per l'esportazione dei simboli
#[unsafe(no_mangle)]
pub unsafe extern "C" fn png_error(
    mut png_ptr: *const png_struct,
    mut error_message: *const libc::c_char,
) {
    exit(1 as libc::c_int);
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn png_warning(
    mut png_ptr: *const png_struct,
    mut warning_message: *const libc::c_char,
) {}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn png_destroy_png_struct(mut png_ptr: *mut png_struct) {
    if !png_ptr.is_null() {
        memset(
            png_ptr as *mut libc::c_void,
            0 as libc::c_int,
            ::core::mem::size_of::<png_struct>() as libc::c_ulong,
        );
        free(png_ptr as *mut libc::c_void);
    }
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn png_malloc_base(
    mut png_ptr: *const png_struct,
    mut size: png_alloc_size_t,
) -> *mut libc::c_void {
    if size > 18446744073709551615 as libc::c_ulong {
        return 0 as *mut libc::c_void;
    }
    return malloc(size);
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn png_calloc(
    mut png_ptr: *const png_struct,
    mut size: png_alloc_size_t,
) -> *mut libc::c_void {
    let mut ret: *mut libc::c_void = png_malloc_base(png_ptr, size);
    if !ret.is_null() {
        memset(ret, 0 as libc::c_int, size);
    }
    return ret;
}

unsafe extern "C" fn png_malloc_array_checked(
    mut png_ptr: *const png_struct,
    mut nelements: libc::c_int,
    mut element_size: size_t,
) -> *mut libc::c_void {
    let mut req: png_alloc_size_t = nelements as png_alloc_size_t;
    if req <= (18446744073709551615 as libc::c_ulong).wrapping_div(element_size) {
        return png_malloc_base(png_ptr, req.wrapping_mul(element_size));
    }
    return 0 as *mut libc::c_void;
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn png_malloc_array(
    mut png_ptr: *const png_struct,
    mut nelements: libc::c_int,
    mut element_size: size_t,
) -> *mut libc::c_void {
    if nelements <= 0 as libc::c_int || element_size == 0 as libc::c_int as libc::c_ulong
    {
        png_error(
            png_ptr,
            b"internal error: array alloc\0" as *const u8 as *const libc::c_char,
        );
    }
    return png_malloc_array_checked(png_ptr, nelements, element_size);
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn png_realloc_array(
    mut png_ptr: *const png_struct,
    mut old_array: *const libc::c_void,
    mut old_elements: libc::c_int,
    mut add_elements: libc::c_int,
    mut element_size: size_t,
) -> *mut libc::c_void {
    if add_elements <= 0 as libc::c_int
        || element_size == 0 as libc::c_int as libc::c_ulong
        || old_elements < 0 as libc::c_int
        || old_array.is_null() && old_elements > 0 as libc::c_int
    {
        png_error(
            png_ptr,
            b"internal error: array realloc\0" as *const u8 as *const libc::c_char,
        );
    }
    if add_elements <= 2147483647 as libc::c_int - old_elements {
        let mut new_array: *mut libc::c_void = png_malloc_array_checked(
            png_ptr,
            old_elements + add_elements,
            element_size,
        );
        if !new_array.is_null() {
            if old_elements > 0 as libc::c_int {
                memcpy(
                    new_array,
                    old_array,
                    element_size
                        .wrapping_mul(old_elements as libc::c_uint as libc::c_ulong),
                );
            }
            memset(
                (new_array as *mut libc::c_char)
                    .offset(
                        element_size
                            .wrapping_mul(old_elements as libc::c_uint as libc::c_ulong)
                            as isize,
                    ) as *mut libc::c_void,
                0 as libc::c_int,
                element_size.wrapping_mul(add_elements as libc::c_uint as libc::c_ulong),
            );
            return new_array;
        }
    }
    return 0 as *mut libc::c_void;
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn png_malloc(
    mut png_ptr: *const png_struct,
    mut size: png_alloc_size_t,
) -> *mut libc::c_void {
    let mut ret: *mut libc::c_void = 0 as *mut libc::c_void;
    if png_ptr.is_null() {
        return 0 as *mut libc::c_void;
    }
    ret = png_malloc_base(png_ptr, size);
    if ret.is_null() {
        png_error(png_ptr, b"Out of memory\0" as *const u8 as *const libc::c_char);
    }
    return ret;
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn png_malloc_warn(
    mut png_ptr: *const png_struct,
    mut size: png_alloc_size_t,
) -> *mut libc::c_void {
    if !png_ptr.is_null() {
        let mut ret: *mut libc::c_void = png_malloc_base(png_ptr, size);
        if !ret.is_null() {
            return ret;
        }
        png_warning(png_ptr, b"Out of memory\0" as *const u8 as *const libc::c_char);
    }
    return 0 as *mut libc::c_void;
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn png_free(
    mut png_ptr: *const png_struct,
    mut ptr: *mut libc::c_void,
) {
    if png_ptr.is_null() || ptr.is_null() {
        return;
    }
    free(ptr);
}

fn main() {}