use std::rc::Rc;
use std::cell::RefCell;
// RefCell<T> allows us to have multiple references to a mutable object,
// in a single threaded way, but is not thread-safe, to provide interior
// mutability.
// It uses Rust's lifetimes to implement 'dynamic borrowing', with the
// borrows being tracked at runtime,
pub struct List<T> {
    head: Link<T>,
    tail: Link<T>
}

type Link<T> = Option<Rc<RefCell<Node<T>>>>;

struct Node<T> {
    elem: T,
    next: Link<T>,
    prev: Link<T>
}

impl Node<T> {
    fn new(elem: T) -> Rc<RefCell<Self>> {
        Rc::new(RefCell::new(Node {
            elem,
            prev: None,
            next: None
        }))
    }
}