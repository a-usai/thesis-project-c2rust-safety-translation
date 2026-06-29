// Red-Black Tree — 100 % Safe Rust, production-ready
// ─────────────────────────────────────────────────────────────────────────────
// Strategia di ownership:
//   • Rc<RefCell<Node>> per i figli (left, right): ownership condivisa che
//     rispecchia fedelmente il doppio puntatore C; RefCell abilita la
//     mutabilità interna a runtime (interior mutability).
//   • Weak<RefCell<Node>> per il puntatore al padre: rompe il ciclo di
//     reference counting (Rc → Rc → … → Rc) che causerebbe memory leak;
//     Weak non incrementa il contatore forte e può essere upgraded a Rc
//     on demand.
//   • Color è un enum idiomatico che rimpiazza gli interi 0 / 1 del C.
//   • Zero unsafe, zero libc, zero malloc / free.
// ─────────────────────────────────────────────────────────────────────────────

use std::cell::RefCell;
use std::rc::{Rc, Weak};

// ── Alias di tipo ─────────────────────────────────────────────────────────────

/// Strong reference a un nodo (proprietà condivisa).
type NodeRef = Rc<RefCell<Node>>;
/// Weak reference per il puntatore al padre (spezza i cicli Rc).
type WeakRef = Weak<RefCell<Node>>;
/// Puntatore opzionale a un figlio — None ≡ NULL del C.
type Link = Option<NodeRef>;

// ── Colore ────────────────────────────────────────────────────────────────────

/// Colore del nodo Red-Black.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Color {
    Red,
    Black,
}

// ── Nodo ──────────────────────────────────────────────────────────────────────

/// Nodo dell'albero Red-Black.
pub struct Node {
    pub data:   i32,
    pub color:  Color,
    /// None se questo nodo è la radice.
    pub parent: Option<WeakRef>,
    pub left:   Link,
    pub right:  Link,
}

impl Node {
    /// Crea un nuovo nodo foglia rosso (i nuovi nodi sono sempre Red).
    fn new(data: i32) -> NodeRef {
        Rc::new(RefCell::new(Node {
            data,
            color:  Color::Red,
            parent: None,
            left:   None,
            right:  None,
        }))
    }
}

// ── Helper di accesso sicuro ──────────────────────────────────────────────────

/// Eleva il Weak del padre a un Rc forte, o None se la radice.
fn get_parent(node: &NodeRef) -> Link {
    node.borrow().parent.as_ref().and_then(|w| w.upgrade())
}

/// Clona il figlio sinistro (None = assente).
fn get_left(node: &NodeRef) -> Link {
    node.borrow().left.clone()
}

/// Clona il figlio destro (None = assente).
fn get_right(node: &NodeRef) -> Link {
    node.borrow().right.clone()
}

/// Legge il colore del nodo.
fn get_color(node: &NodeRef) -> Color {
    node.borrow().color
}

/// Imposta il colore del nodo.
fn set_color(node: &NodeRef, color: Color) {
    node.borrow_mut().color = color;
}

/// Restituisce `true` se `link` punta allo stesso nodo Rc di `target`.
fn is_same(link: &Link, target: &NodeRef) -> bool {
    link.as_ref().is_some_and(|n| Rc::ptr_eq(n, target))
}

// ── Inserimento BST ───────────────────────────────────────────────────────────

/// Inserisce `temp` nell'albero BST radicato in `trav`.
/// Imposta i puntatori parent dopo ogni chiamata ricorsiva.
/// Restituisce la radice del sotto-albero risultante.
fn bst(trav: Link, temp: NodeRef) -> NodeRef {
    match trav {
        None => temp,
        Some(node) => {
            // Estrai i dati prima di qualsiasi mutuo prestito
            let temp_data = temp.borrow().data;
            let node_data = node.borrow().data;

            if temp_data < node_data {
                let new_left = bst(get_left(&node), temp);
                new_left.borrow_mut().parent = Some(Rc::downgrade(&node));
                node.borrow_mut().left = Some(new_left);
            } else if temp_data > node_data {
                let new_right = bst(get_right(&node), temp);
                new_right.borrow_mut().parent = Some(Rc::downgrade(&node));
                node.borrow_mut().right = Some(new_right);
            }
            node
        }
    }
}

// ── Rotazioni ─────────────────────────────────────────────────────────────────

/// Rotazione destra su `temp`; aggiorna `root` se `temp` era la radice.
fn right_rotate(root: &mut Link, temp: &NodeRef) {
    let left = get_left(temp).expect("right_rotate: nodo senza figlio sinistro");

    // temp->l = left->r
    let left_right = get_right(&left);
    temp.borrow_mut().left = left_right.clone();

    // if (temp->l != NULL) temp->l->p = temp
    if let Some(ref lr) = left_right {
        lr.borrow_mut().parent = Some(Rc::downgrade(temp));
    }

    // left->p = temp->p
    let temp_parent_weak = temp.borrow().parent.clone();
    left.borrow_mut().parent = temp_parent_weak.clone();

    // if (temp->p == NULL) root = left; else aggiorna il link del nonno
    match temp_parent_weak.and_then(|w| w.upgrade()) {
        None => *root = Some(left.clone()),
        Some(ref parent) => {
            let parent_left = parent.borrow().left.clone();
            if is_same(&parent_left, temp) {
                parent.borrow_mut().left = Some(left.clone());
            } else {
                parent.borrow_mut().right = Some(left.clone());
            }
        }
    }

    // left->r = temp; temp->p = left
    left.borrow_mut().right = Some(temp.clone());
    temp.borrow_mut().parent = Some(Rc::downgrade(&left));
}

/// Rotazione sinistra su `temp`; aggiorna `root` se `temp` era la radice.
fn left_rotate(root: &mut Link, temp: &NodeRef) {
    let right = get_right(temp).expect("left_rotate: nodo senza figlio destro");

    // temp->r = right->l
    let right_left = get_left(&right);
    temp.borrow_mut().right = right_left.clone();

    // if (temp->r != NULL) temp->r->p = temp
    if let Some(ref rl) = right_left {
        rl.borrow_mut().parent = Some(Rc::downgrade(temp));
    }

    // right->p = temp->p
    let temp_parent_weak = temp.borrow().parent.clone();
    right.borrow_mut().parent = temp_parent_weak.clone();

    // if (temp->p == NULL) root = right; else aggiorna il link del nonno
    match temp_parent_weak.and_then(|w| w.upgrade()) {
        None => *root = Some(right.clone()),
        Some(ref parent) => {
            let parent_left = parent.borrow().left.clone();
            if is_same(&parent_left, temp) {
                parent.borrow_mut().left = Some(right.clone());
            } else {
                parent.borrow_mut().right = Some(right.clone());
            }
        }
    }

    // right->l = temp; temp->p = right
    right.borrow_mut().left = Some(temp.clone());
    temp.borrow_mut().parent = Some(Rc::downgrade(&right));
}

// ── Fix-up ────────────────────────────────────────────────────────────────────

/// Ripristina le proprietà Red-Black dopo un inserimento BST.
///
/// Traduzione iterativa (loop) del fixup ricorsivo originale in C.
/// Gestisce i 3 casi standard (Case 1 recolor, Case 2 rotation,
/// Case 3 rotation+recolor) per entrambe le simmetrie (padre sx / padre dx).
fn fixup(root: &mut Link, initial_pt: &NodeRef) {
    let mut pt = initial_pt.clone();

    loop {
        // Condizione uscita 1: pt è la radice
        if root.as_ref().is_some_and(|r| Rc::ptr_eq(r, &pt)) {
            break;
        }
        // Condizione uscita 2: pt è nero
        if get_color(&pt) == Color::Black {
            break;
        }
        // Ottieni il padre (assenza → break)
        let parent = match get_parent(&pt) {
            Some(p) => p,
            None => break,
        };
        // Condizione uscita 3: il padre è nero (proprietà rispettata)
        if get_color(&parent) == Color::Black {
            break;
        }
        // Ottieni il nonno (assenza → break)
        let gp = match get_parent(&parent) {
            Some(g) => g,
            None => break,
        };

        if is_same(&get_left(&gp), &parent) {
            // ── Padre è figlio SINISTRO del nonno ────────────────────────────
            let uncle = get_right(&gp);

            match &uncle {
                Some(u) if u.borrow().color == Color::Red => {
                    // Case 1: zio rosso → recolor e sali
                    set_color(&gp, Color::Red);
                    set_color(&parent, Color::Black);
                    set_color(u, Color::Black);
                    pt = gp.clone();
                }
                _ => {
                    // Case 2: pt è figlio destro → rotazione sinistra su padre
                    if is_same(&get_right(&parent), &pt) {
                        left_rotate(root, &parent);
                        pt = parent.clone();
                    }
                    // Case 3: rotazione destra sul nonno + scambio colori
                    let new_parent = match get_parent(&pt) {
                        Some(p) => p,
                        None => break,
                    };
                    let new_gp = match get_parent(&new_parent) {
                        Some(g) => g,
                        None => break,
                    };
                    let np_col = get_color(&new_parent);
                    let gp_col = get_color(&new_gp);
                    set_color(&new_parent, gp_col);
                    set_color(&new_gp, np_col);
                    right_rotate(root, &new_gp);
                    pt = new_parent.clone();
                }
            }
        } else {
            // ── Padre è figlio DESTRO del nonno (simmetrico) ─────────────────
            let uncle = get_left(&gp);

            match &uncle {
                Some(u) if u.borrow().color == Color::Red => {
                    // Case 1: zio rosso → recolor e sali
                    set_color(&gp, Color::Red);
                    set_color(&parent, Color::Black);
                    set_color(u, Color::Black);
                    pt = gp.clone();
                }
                _ => {
                    // Case 2: pt è figlio sinistro → rotazione destra su padre
                    if is_same(&get_left(&parent), &pt) {
                        right_rotate(root, &parent);
                        pt = parent.clone();
                    }
                    // Case 3: rotazione sinistra sul nonno + scambio colori
                    let new_parent = match get_parent(&pt) {
                        Some(p) => p,
                        None => break,
                    };
                    let new_gp = match get_parent(&new_parent) {
                        Some(g) => g,
                        None => break,
                    };
                    let np_col = get_color(&new_parent);
                    let gp_col = get_color(&new_gp);
                    set_color(&new_parent, gp_col);
                    set_color(&new_gp, np_col);
                    left_rotate(root, &new_gp);
                    pt = new_parent.clone();
                }
            }
        }
    }
}

// ── Visita in-order ───────────────────────────────────────────────────────────

/// Stampa i valori dell'albero in ordine crescente (visita simmetrica).
fn inorder(trav: &Link) {
    if let Some(node) = trav {
        let left  = get_left(node);
        let data  = node.borrow().data;
        let right = get_right(node);
        inorder(&left);
        print!("{data} ");
        inorder(&right);
    }
}

// ── Entry point ───────────────────────────────────────────────────────────────

fn main() {
    let values = [7_i32, 6, 5, 4, 3, 2, 1];
    let mut root: Link = None;

    for &v in &values {
        let temp = Node::new(v);
        // bst consuma root (move), restituisce la nuova radice
        root = Some(bst(root, temp.clone()));
        fixup(&mut root, &temp);
        // La radice deve essere sempre nera
        if let Some(ref r) = root {
            r.borrow_mut().color = Color::Black;
        }
    }

    println!("Inorder Traversal of Created Tree");
    inorder(&root);
    println!();
}