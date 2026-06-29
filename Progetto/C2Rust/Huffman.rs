//cargo clippy > report_clippy.txt 2>&1 generazione testuale
//cargo clippy --message-format=json 2>/dev/null | python clippy_parser.py verifica tramite script e generazione tabella
//cargo clippy --message-format=json 2>&1 | tee clippy_results.json PRIMO DA AVVIARE DAL TERMINALE
//python3 clippy_parser.py COMANDO SUCCESSIVO
//#![allow(dead_code, mutable_transmutes, non_camel_case_types, non_snake_case, non_upper_case_globals, unused_assignments, unused_mut)]
#[derive(Copy, Clone)]
#[repr(C)]
pub struct MinHeapNode {
    pub data: libc::c_char,
    pub freq: libc::c_uint,
    pub left: *mut MinHeapNode,
    pub right: *mut MinHeapNode,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct MinHeap {
    pub size: libc::c_uint,
    pub capacity: libc::c_uint,
    pub array: *mut *mut MinHeapNode,
}

unsafe fn swapMinHeapNode(a: *mut *mut MinHeapNode, b: *mut *mut MinHeapNode) {}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn minHeapify(mut minHeap: *mut MinHeap, mut idx: libc::c_int) {
    let mut smallest: libc::c_int = idx;
    let mut left: libc::c_int = 2 as libc::c_int * idx + 1 as libc::c_int;
    let mut right: libc::c_int = 2 as libc::c_int * idx + 2 as libc::c_int;
    if (left as libc::c_uint) < (*minHeap).size
        && (**((*minHeap).array).offset(left as isize)).freq
            < (**((*minHeap).array).offset(smallest as isize)).freq
    {
        smallest = left;
    }
    if (right as libc::c_uint) < (*minHeap).size
        && (**((*minHeap).array).offset(right as isize)).freq
            < (**((*minHeap).array).offset(smallest as isize)).freq
    {
        smallest = right;
    }
    if smallest != idx {
        swapMinHeapNode(
            &mut *((*minHeap).array).offset(smallest as isize),
            &mut *((*minHeap).array).offset(idx as isize),
        );
        minHeapify(minHeap, smallest);
    }
}
#[unsafe(no_mangle)]
pub unsafe extern "C" fn extractMin(mut minHeap: *mut MinHeap) -> *mut MinHeapNode {
    let mut temp: *mut MinHeapNode = *((*minHeap).array)
        .offset(0 as libc::c_int as isize);
    let ref mut fresh0 = *((*minHeap).array).offset(0 as libc::c_int as isize);
    *fresh0 = *((*minHeap).array)
        .offset(
            ((*minHeap).size).wrapping_sub(1 as libc::c_int as libc::c_uint) as isize,
        );
    (*minHeap).size = ((*minHeap).size).wrapping_sub(1);
    (*minHeap).size;
    minHeapify(minHeap, 0 as libc::c_int);
    return temp;
}

fn main() {
    // Lasciala vuota, serve solo a far felice il compilatore
}