unsafe extern "C" {
    fn printf(_: *const libc::c_char, _: ...) -> libc::c_int;
}
#[unsafe(no_mangle)]
pub unsafe extern "C" fn swap(mut a: *mut libc::c_int, mut b: *mut libc::c_int) {
    let mut temp: libc::c_int = *a;
    *a = *b;
    *b = temp;
}
#[unsafe(no_mangle)]
pub unsafe extern "C" fn partition(
    mut arr: *mut libc::c_int,
    mut low: libc::c_int,
    mut high: libc::c_int,
) -> libc::c_int {
    let mut p: libc::c_int = *arr.offset(low as isize);
    let mut i: libc::c_int = low;
    let mut j: libc::c_int = high;
    while i < j {
        while *arr.offset(i as isize) <= p && i <= high - 1 as libc::c_int {
            i += 1;
            i;
        }
        while *arr.offset(j as isize) > p && j >= low + 1 as libc::c_int {
            j -= 1;
            j;
        }
        if i < j {
            swap(&mut *arr.offset(i as isize), &mut *arr.offset(j as isize));
        }
    }
    swap(&mut *arr.offset(low as isize), &mut *arr.offset(j as isize));
    return j;
}
#[unsafe(no_mangle)]
pub unsafe extern "C" fn quickSort(
    mut arr: *mut libc::c_int,
    mut low: libc::c_int,
    mut high: libc::c_int,
) {
    if low < high {
        let mut pi: libc::c_int = partition(arr, low, high);
        quickSort(arr, low, pi - 1 as libc::c_int);
        quickSort(arr, pi + 1 as libc::c_int, high);
    }
}
unsafe fn main_0() -> libc::c_int {
    let mut arr: [libc::c_int; 5] = [
        4 as libc::c_int,
        2 as libc::c_int,
        5 as libc::c_int,
        3 as libc::c_int,
        1 as libc::c_int,
    ];
    let mut n: libc::c_int = (::core::mem::size_of::<[libc::c_int; 5]>()
        as libc::c_ulong)
        .wrapping_div(::core::mem::size_of::<libc::c_int>() as libc::c_ulong)
        as libc::c_int;
    quickSort(arr.as_mut_ptr(), 0 as libc::c_int, n - 1 as libc::c_int);
    let mut i: libc::c_int = 0 as libc::c_int;
    while i < n {
        printf(b"%d \0" as *const u8 as *const libc::c_char, arr[i as usize]);
        i += 1;
        i;
    }
    return 0 as libc::c_int;
}
pub fn main() {
    unsafe { ::std::process::exit(main_0() as i32) }
}