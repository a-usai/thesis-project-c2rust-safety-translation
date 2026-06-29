#![allow(dead_code, mutable_transmutes, non_camel_case_types, non_snake_case, non_upper_case_globals, unused_assignments, unused_mut)]
extern "C" {
    fn printf(_: *const libc::c_char, _: ...) -> libc::c_int;
    fn sprintf(_: *mut libc::c_char, _: *const libc::c_char, _: ...) -> libc::c_int;
    fn malloc(_: libc::c_ulong) -> *mut libc::c_void;
    fn free(_: *mut libc::c_void);
    fn strcpy(_: *mut libc::c_char, _: *const libc::c_char) -> *mut libc::c_char;
    fn strncat(
        _: *mut libc::c_char,
        _: *const libc::c_char,
        _: libc::c_ulong,
    ) -> *mut libc::c_char;
    fn strcmp(_: *const libc::c_char, _: *const libc::c_char) -> libc::c_int;
    fn strlen(_: *const libc::c_char) -> libc::c_ulong;
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct DictEntry {
    pub sequence: *mut libc::c_char,
    pub code: libc::c_int,
}
#[no_mangle]
pub static mut dictionary: [DictEntry; 4096] = [DictEntry {
    sequence: 0 as *const libc::c_char as *mut libc::c_char,
    code: 0,
}; 4096];
#[no_mangle]
pub static mut dict_size: libc::c_int = 256 as libc::c_int;
#[no_mangle]
pub unsafe extern "C" fn initializeTable() {
    let mut i: libc::c_int = 0 as libc::c_int;
    while i < dict_size {
        dictionary[i as usize]
            .sequence = malloc(2 as libc::c_int as libc::c_ulong) as *mut libc::c_char;
        sprintf(
            dictionary[i as usize].sequence,
            b"%c\0" as *const u8 as *const libc::c_char,
            i,
        );
        dictionary[i as usize].code = i;
        i += 1;
        i;
    }
}
#[no_mangle]
pub unsafe extern "C" fn findCode(mut str: *mut libc::c_char) -> libc::c_int {
    let mut i: libc::c_int = 0 as libc::c_int;
    while i < dict_size {
        if strcmp(dictionary[i as usize].sequence, str) == 0 as libc::c_int {
            return dictionary[i as usize].code;
        }
        i += 1;
        i;
    }
    return -(1 as libc::c_int);
}
#[no_mangle]
pub unsafe extern "C" fn addEntry(mut str: *mut libc::c_char) {
    if dict_size < 4096 as libc::c_int {
        dictionary[dict_size as usize]
            .sequence = malloc(
            (strlen(str)).wrapping_add(1 as libc::c_int as libc::c_ulong),
        ) as *mut libc::c_char;
        strcpy(dictionary[dict_size as usize].sequence, str);
        dictionary[dict_size as usize].code = dict_size;
        dict_size += 1;
        dict_size;
    }
}
#[no_mangle]
pub unsafe extern "C" fn compress(mut input: *mut libc::c_char) {
    let mut current: [libc::c_char; 1000] = *::core::mem::transmute::<
        &[u8; 1000],
        &mut [libc::c_char; 1000],
    >(
        b"\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0",
    );
    let mut next: libc::c_char = 0;
    let mut code: libc::c_int = 0;
    let mut i: libc::c_int = 0 as libc::c_int;
    while (i as libc::c_ulong) < strlen(input) {
        next = *input.offset(i as isize);
        let mut temp: [libc::c_char; 1000] = [0; 1000];
        strcpy(temp.as_mut_ptr(), current.as_mut_ptr());
        strncat(temp.as_mut_ptr(), &mut next, 1 as libc::c_int as libc::c_ulong);
        if findCode(temp.as_mut_ptr()) != -(1 as libc::c_int) {
            strcpy(current.as_mut_ptr(), temp.as_mut_ptr());
        } else {
            code = findCode(current.as_mut_ptr());
            printf(b"%d \0" as *const u8 as *const libc::c_char, code);
            addEntry(temp.as_mut_ptr());
            strcpy(current.as_mut_ptr(), &mut next);
        }
        i += 1;
        i;
    }
    code = findCode(current.as_mut_ptr());
    printf(b"%d\n\0" as *const u8 as *const libc::c_char, code);
}
#[no_mangle]
pub unsafe extern "C" fn freeTable() {
    let mut i: libc::c_int = 0 as libc::c_int;
    while i < dict_size {
        free(dictionary[i as usize].sequence as *mut libc::c_void);
        i += 1;
        i;
    }
}
unsafe fn main_0() -> libc::c_int {
    let mut input: [libc::c_char; 25] = *::core::mem::transmute::<
        &[u8; 25],
        &mut [libc::c_char; 25],
    >(b"TOBEORNOTTOBEORTOBEORNOT\0");
    initializeTable();
    compress(input.as_mut_ptr());
    freeTable();
    return 0 as libc::c_int;
}
pub fn main() {
    unsafe { ::std::process::exit(main_0() as i32) }
}
