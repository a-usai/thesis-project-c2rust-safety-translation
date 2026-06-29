unsafe extern "C" {
    fn printf(_: *const libc::c_char, _: ...) -> libc::c_int;
    fn malloc(_: libc::c_ulong) -> *mut libc::c_void;
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct node {
    pub d: libc::c_int,
    pub c: libc::c_int,
    pub p: *mut node,
    pub r: *mut node,
    pub l: *mut node,
}
#[unsafe(no_mangle)]
pub static mut root: *mut node = 0 as *const node as *mut node;

#[unsafe(no_mangle)]
pub unsafe extern "C" fn bst(mut trav: *mut node, mut temp: *mut node) -> *mut node {
    if trav.is_null() {
        return temp;
    }
    if (*temp).d < (*trav).d {
        (*trav).l = bst((*trav).l, temp);
        (*(*trav).l).p = trav;
    } else if (*temp).d > (*trav).d {
        (*trav).r = bst((*trav).r, temp);
        (*(*trav).r).p = trav;
    }
    return trav;
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn rightrotate(mut temp: *mut node) {
    let mut left: *mut node = (*temp).l;
    (*temp).l = (*left).r;
    if !((*temp).l).is_null() {
        (*(*temp).l).p = temp;
    }
    (*left).p = (*temp).p;
    if ((*temp).p).is_null() {
        root = left;
    } else if temp == (*(*temp).p).l {
        (*(*temp).p).l = left;
    } else {
        (*(*temp).p).r = left;
    }
    (*left).r = temp;
    (*temp).p = left;
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn leftrotate(mut temp: *mut node) {
    let mut right: *mut node = (*temp).r;
    (*temp).r = (*right).l;
    if !((*temp).r).is_null() {
        (*(*temp).r).p = temp;
    }
    (*right).p = (*temp).p;
    if ((*temp).p).is_null() {
        root = right;
    } else if temp == (*(*temp).p).l {
        (*(*temp).p).l = right;
    } else {
        (*(*temp).p).r = right;
    }
    (*right).l = temp;
    (*temp).p = right;
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn fixup(mut root_0: *mut node, mut pt: *mut node) {
    let mut parent_pt: *mut node = 0 as *mut node;
    let mut grand_parent_pt: *mut node = 0 as *mut node;
    while pt != root_0 && (*pt).c != 0 as libc::c_int && (*(*pt).p).c == 1 as libc::c_int
    {
        parent_pt = (*pt).p;
        grand_parent_pt = (*(*pt).p).p;
        if parent_pt == (*grand_parent_pt).l {
            let mut uncle_pt: *mut node = (*grand_parent_pt).r;
            if !uncle_pt.is_null() && (*uncle_pt).c == 1 as libc::c_int {
                (*grand_parent_pt).c = 1 as libc::c_int;
                (*parent_pt).c = 0 as libc::c_int;
                (*uncle_pt).c = 0 as libc::c_int;
                pt = grand_parent_pt;
            } else {
                if pt == (*parent_pt).r {
                    leftrotate(parent_pt);
                    pt = parent_pt;
                    parent_pt = (*pt).p;
                }
                rightrotate(grand_parent_pt);
                let mut t: libc::c_int = (*parent_pt).c;
                (*parent_pt).c = (*grand_parent_pt).c;
                (*grand_parent_pt).c = t;
                pt = parent_pt;
            }
        } else {
            let mut uncle_pt_0: *mut node = (*grand_parent_pt).l;
            if !uncle_pt_0.is_null() && (*uncle_pt_0).c == 1 as libc::c_int {
                (*grand_parent_pt).c = 1 as libc::c_int;
                (*parent_pt).c = 0 as libc::c_int;
                (*uncle_pt_0).c = 0 as libc::c_int;
                pt = grand_parent_pt;
            } else {
                if pt == (*parent_pt).l {
                    rightrotate(parent_pt);
                    pt = parent_pt;
                    parent_pt = (*pt).p;
                }
                leftrotate(grand_parent_pt);
                let mut t_0: libc::c_int = (*parent_pt).c;
                (*parent_pt).c = (*grand_parent_pt).c;
                (*grand_parent_pt).c = t_0;
                pt = parent_pt;
            }
        }
    }
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn inorder(mut trav: *mut node) {
    if trav.is_null() {
        return;
    }
    inorder((*trav).l);
    printf(b"%d \0" as *const u8 as *const libc::c_char, (*trav).d);
    inorder((*trav).r);
}

unsafe fn main_0() -> libc::c_int {
    let mut n: libc::c_int = 7 as libc::c_int;
    let mut a: [libc::c_int; 7] = [
        7 as libc::c_int,
        6 as libc::c_int,
        5 as libc::c_int,
        4 as libc::c_int,
        3 as libc::c_int,
        2 as libc::c_int,
        1 as libc::c_int,
    ];
    let mut i: libc::c_int = 0 as libc::c_int;
    while i < n {
        let mut temp: *mut node = malloc(::core::mem::size_of::<node>() as libc::c_ulong)
            as *mut node;
        (*temp).r = 0 as *mut node;
        (*temp).l = 0 as *mut node;
        (*temp).p = 0 as *mut node;
        (*temp).d = a[i as usize];
        (*temp).c = 1 as libc::c_int;
        root = bst(root, temp);
        fixup(root, temp);
        (*root).c = 0 as libc::c_int;
        i += 1;
        i;
    }
    printf(b"Inorder Traversal of Created Tree\n\0" as *const u8 as *const libc::c_char);
    inorder(root);
    return 0 as libc::c_int;
}

pub fn main() {
    unsafe { ::std::process::exit(main_0() as i32) }
}