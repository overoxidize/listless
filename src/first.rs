use std::mem;
struct Node<T> {
    // 
    elem: T,
    next: Link<T>,
}
type Link<T> = Option<Box<Node<T>>>;
// Box<T> is a `smart pointer` in rust, which will help for situations when
// we want to use a type whose size can't be predicted at compile time.
// Box allows us to use 'indirection', by allowing the compiler to refer
// to a *pointer* to the value of unknown size, which lives on the heap,
// allowing for the pointer to be passed around.
pub struct List<T> {
    head: Link<T>,
}
// layout 1:

// [Elem A, ptr] -> (Elem B, ptr) -> (Elem C, ptr) -> (Empty *junk*)

// [Elem A, ptr] -> (Elem B, ptr) -> (Empty *junk*)
// [Elem C, ptr] -> (Empty *junk*)

// split off C: This forces us to go from (Elem C, ptr) -> [Elem C, ptr],
// which requires us to move C from the heap, to the stack, sullying the advantage
// of being able to move lists and elements around solely by moving pointers.

// layout 2:

// [ptr] -> (Elem A, ptr) -> (Elem B, ptr) -> (Elem C, *null*)

// split off C: Here, only the pointer to C is moved, from the stack to the
// heap, allowing  for C to remain on the heap.

// [ptr] -> (Elem A, ptr) -> (Elem B, *null*)
// [ptr] -> (Elem C, *null*)


pub struct IntoIter<T>(List<T>);
    // Collections are iterated via the Iterator trait, with traits being somewhat
    // analgous to interfaces in languages like C++ and Java.
    // Traits like this have an *associated type*, Item, which is the type,
    // that will be returned when next is called.

    // There are three types of iterator each collection could possible make

    // IntoIter - T
    // IterMut - &mut T
    // Iter - &T

    // Note: this is an example of the `newtype` pattern,
impl<T> Iterator for IntoIter<T> {
    type Item = T;
    fn next(&mut self) -> Option<Self::Item> {
        self.0.pop()
    }
}

pub struct IterMut<'a, T> {
    next: Option<&'a mut Node<T>>
}

impl<T> List<T> {
    pub fn iter_mut(&mut self) -> IterMut<'_, T> {
        IterMut { next: self.head.as_deref_mut() }
    }
}

impl<'a, T> Iterator for IterMut<'a, T> {
    type Item = &'a mut T;

    fn next(&mut self) -> Option<Self::Item> {
        self.next.take().map(|node| {
            self.next = node.next.as_deref_mut();
            &mut node.elem
        })
    }
}
pub struct Iter<'a, T> {
    // 'a is an example of a `lifetime`, which "names" a region of code,
    // and when we tag a reference with that lifetime, i.e `&'a foo`, we're saying that
    // foo must live as long as the region named 'a.

    // We can avoid explicitly writing lifetimes in some situations, where the compiler
    // will pick the lifetimes for you, which is known as *lifetime elision*.

    // If there's only one input reference, the output must be
    // derived from that input:
        // fn foo(&A) -> &B => fn foo<'a>(&'a A) -> &'a B;

    // If there are many inputs, assume they're all independent:
        // fn foo(&A, &B, &C) => fn foo<'a, 'b, 'c>(&'a A, &'b B, &'c C);
    
    // In methods, assume all output lifetimes are derived from the lifetime of self.
        // fn foo(&self, &B, &C) => fn foo<'a, 'b, 'c>(&'a self, &'b B, &'c C);
        
    next: Option<&'a Node<T>>,
}


impl<'a, T> Iterator for Iter<'a, T> {
    type Item = &'a T;

    fn next(&mut self) -> Option<Self::Item> {
        self.next.map(|node| {
            self.next = node.next.as_deref();
            // We could say node.next.as_ref().map::<&Node<T>, _>(|node| &node),
            // since map is generic, and the turbofish `::<>`, lets us instruct
            // the compiler on our expectations, so that it uses deref coercion
            // as opposed to us dereferencing pointers manually.
            &node.elem
        })
    }
}
impl<T> List<T> {
    pub fn iter<'a>(&'a self) -> Iter<'a, T> {
        // We could implicitly elide this lifetime, or manually do so
        // i.e <'_>, and let the compiler infer the necessary lifetime.
        Iter { next: self.head.as_deref()}
    }
    pub fn into_iter(self) -> IntoIter<T> {
        IntoIter(self)
    }
    pub fn new() -> Self {
        List { head: None }
    }

    pub fn push(&mut self, elem: T) {
        let new_node = Box::new(Node {
            elem,
            next: self.head.take(),
        });

        self.head = Some(new_node);
    }

    pub fn pop(&mut self) -> Option<T> {
        self.head.take().map(|node| {
            self.head = node.next;
            node.elem
        })
    }

    pub fn peek(&self) -> Option<&T> {
        self.head.as_ref().map(|node| &node.elem)
    }

    pub fn peek_mut(&mut self) -> Option<&mut T> {
        self.head.as_mut().map(|node| &mut node.elem)
    }
}

impl<T> Drop for List<T> {
    fn drop(&mut self) {
        let mut cur_link = self.head.take();

        while let Some(mut boxed_node) = cur_link {
            cur_link = boxed_node.next.take();
        }
    }
}

#[cfg(test)]
mod test {
    use super::List;

    #[test]
    fn basics() {
        let mut list = List::new();

        // Check empty list behaves right
        assert_eq!(list.pop(), None);

        // Populate list
        list.push(1);
        list.push(2);
        list.push(3);

        // Check normal removal
        assert_eq!(list.pop(), Some(3));
        assert_eq!(list.pop(), Some(2));

        // Push some more just to make sure nothing's corrupted
        list.push(4);
        list.push(5);

        // Check normal removal
        assert_eq!(list.pop(), Some(5));
        assert_eq!(list.pop(), Some(4));

        // Check exhaustion
        assert_eq!(list.pop(), Some(1));
        assert_eq!(list.pop(), None);
    }

    #[test]
    fn peek() {
        let mut list = List::new();
        assert_eq!(list.peek(), None);
        assert_eq!(list.peek_mut(), None);
        list.push(1);
        list.push(2);
        list.push(3);

        assert_eq!(list.peek(), Some(&3));
        assert_eq!(list.peek_mut(), Some(&mut 3));

        list.peek_mut().map(|value| {
            *value = 42
        });
    
        assert_eq!(list.peek(), Some(&42));
        assert_eq!(list.pop(), Some(42));
    }

    #[test]
    fn into_iter() {
        let mut list = List::new();
        list.push(1); list.push(2); list.push(3);

        let mut iter = list.into_iter();
        assert_eq!(iter.next(), Some(3));
        assert_eq!(iter.next(), Some(2));
        assert_eq!(iter.next(), Some(1));
        assert_eq!(iter.next(), None);
    }

    #[test]
    fn iter() {
        let mut list = List::new();
        list.push(1); list.push(2); list.push(3);

        let mut iter = list.iter();
        assert_eq!(iter.next(), Some(&3));
        assert_eq!(iter.next(), Some(&2));
        assert_eq!(iter.next(), Some(&1));
    }

    #[test]
    fn iter_mut() {
        let mut list = List::new();
        list.push(1); list.push(2); list.push(3);

        let mut iter = list.iter_mut();
        assert_eq!(iter.next(), Some(&mut 3));
        assert_eq!(iter.next(), Some(&mut 2));
        assert_eq!(iter.next(), Some(&mut 1));
    }
}
fn main() {
    println!("");
}
