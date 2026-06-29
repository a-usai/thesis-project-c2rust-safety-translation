unsafe extern "C" {
    fn printf(_: *const libc::c_char, _: ...) -> libc::c_int;
    fn scanf(_: *const libc::c_char, _: ...) -> libc::c_int;
    fn qsort(
        __base: *mut libc::c_void,
        __nmemb: size_t,
        __size: size_t,
        __compar: __compar_fn_t,
    );
}
pub type size_t = libc::c_ulong;
pub type __compar_fn_t = Option::<
    unsafe extern "C" fn(*const libc::c_void, *const libc::c_void) -> libc::c_int,
>;
#[unsafe(no_mangle)]
pub static mut mat: [[libc::c_int; 20]; 20] = [[0; 20]; 20];
#[unsafe(no_mangle)]
pub static mut V: libc::c_int = 0;
#[unsafe(no_mangle)]
pub static mut dist: [libc::c_int; 20] = [0; 20];
#[unsafe(no_mangle)]
pub static mut q: [libc::c_int; 20] = [0; 20];
#[unsafe(no_mangle)]
pub static mut qp: libc::c_int = 0 as libc::c_int;

#[unsafe(no_mangle)]
pub unsafe extern "C" fn enqueue(mut v: libc::c_int) {
    let fresh0 = qp;
    qp = qp + 1;
    q[fresh0 as usize] = v;
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn cf(
    mut a: *mut libc::c_void,
    mut b: *mut libc::c_void,
) -> libc::c_int {
    let mut x: *mut libc::c_int = a as *mut libc::c_int;
    let mut y: *mut libc::c_int = b as *mut libc::c_int;
    return *y - *x;
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn dequeue() -> libc::c_int {
    qsort(
        q.as_mut_ptr() as *mut libc::c_void,
        qp as size_t,
        ::core::mem::size_of::<libc::c_int>() as libc::c_ulong,
        ::core::mem::transmute::<
            Option::<
                unsafe extern "C" fn(*mut libc::c_void, *mut libc::c_void) -> libc::c_int,
            >,
            __compar_fn_t,
        >(
            Some(
                cf
                    as unsafe extern "C" fn(
                        *mut libc::c_void,
                        *mut libc::c_void,
                    ) -> libc::c_int,
            ),
        ),
    );
    qp -= 1;
    return q[qp as usize];
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn queue_has_something() -> libc::c_int {
    return (qp > 0 as libc::c_int) as libc::c_int;
}

#[unsafe(no_mangle)]
pub static mut visited: [libc::c_int; 20] = [0; 20];
#[unsafe(no_mangle)]
pub static mut vp: libc::c_int = 0 as libc::c_int;

#[unsafe(no_mangle)]
pub unsafe extern "C" fn dijkstra(mut s: libc::c_int) {
    dist[s as usize] = 0 as libc::c_int;
    let mut i: libc::c_int = 0;
    i = 0 as libc::c_int;
    while i < V {
        if i != s {
            dist[i as usize] = 999 as libc::c_int;
        }
        enqueue(i);
        i += 1;
        i;
    }
    while queue_has_something() != 0 {
        let mut u: libc::c_int = dequeue();
        let fresh1 = vp;
        vp = vp + 1;
        visited[fresh1 as usize] = u;
        i = 0 as libc::c_int;
        while i < V {
            if mat[u as usize][i as usize] != 0 {
                if dist[i as usize] > dist[u as usize] + mat[u as usize][i as usize] {
                    dist[i as usize] = dist[u as usize] + mat[u as usize][i as usize];
                }
            }
            i += 1;
            i;
        }
    }
}

unsafe fn main_0(
    mut argc: libc::c_int,
    mut argv: *mut *const libc::c_char,
) -> libc::c_int {
    printf(b"Enter the number of vertices: \0" as *const u8 as *const libc::c_char);
    scanf(b" %d\0" as *const u8 as *const libc::c_char, &mut V as *mut libc::c_int);
    printf(b"Enter the adj matrix: \0" as *const u8 as *const libc::c_char);
    let mut i: libc::c_int = 0;
    let mut j: libc::c_int = 0;
    i = 0 as libc::c_int;
    while i < V {
        j = 0 as libc::c_int;
        while j < V {
            scanf(
                b" %d\0" as *const u8 as *const libc::c_char,
                &mut *(*mat.as_mut_ptr().offset(i as isize))
                    .as_mut_ptr()
                    .offset(j as isize) as *mut libc::c_int,
            );
            j += 1;
            j;
        }
        i += 1;
        i;
    }
    dijkstra(0 as libc::c_int);
    printf(b"\nNode\tDist\n\0" as *const u8 as *const libc::c_char);
    i = 0 as libc::c_int;
    while i < V {
        printf(b"%d\t%d\n\0" as *const u8 as *const libc::c_char, i, dist[i as usize]);
        i += 1;
        i;
    }
    return 0 as libc::c_int;
}

pub fn main() {
    let mut args: Vec::<*mut libc::c_char> = Vec::new();
    for arg in ::std::env::args() {
        args.push(
            (::std::ffi::CString::new(arg))
                .expect("Failed to convert argument into CString.")
                .into_raw(),
        );
    }
    args.push(::core::ptr::null_mut());
    unsafe {
        ::std::process::exit(
            main_0(
                (args.len() - 1) as libc::c_int,
                args.as_mut_ptr() as *mut *const libc::c_char,
            ) as i32,
        )
    }
}