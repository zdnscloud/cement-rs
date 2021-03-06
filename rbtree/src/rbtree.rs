use std::cmp::Ord;
use std::cmp::Ordering;
use std::fmt::{self, Debug};
use std::iter::{FromIterator, IntoIterator};
use std::marker;
use std::mem;
use std::ops::Index;

use crate::rbnode::{Color, NodePtr};

pub struct RBTree<K: Ord, V> {
    root: NodePtr<K, V>,
    len: usize,
}

impl<K: Ord, V> Drop for RBTree<K, V> {
    fn drop(&mut self) {
        self.clear();
    }
}

impl<K: Ord + Clone, V: Clone> Clone for RBTree<K, V> {
    fn clone(&self) -> RBTree<K, V> {
        unsafe {
            let mut new = RBTree::new();
            new.root = self.root.deep_clone();
            new.len = self.len;
            new
        }
    }
}

impl<K, V> Debug for RBTree<K, V>
where
    K: Ord + Debug,
    V: Debug,
{
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_map().entries(self.iter()).finish()
    }
}

impl<K: Ord, V> Default for RBTree<K, V> {
    fn default() -> Self {
        Self::new()
    }
}

impl<K: Ord + Debug, V: Debug> RBTree<K, V> {
    fn tree_print(&self, node: NodePtr<K, V>, direction: i32) {
        if node.is_null() {
            return;
        }
        if direction == 0 {
            unsafe {
                println!("'{:?}' is root node", (*node.0));
            }
        } else {
            let direct = if direction == -1 { "left" } else { "right" };
            unsafe {
                println!(
                    "{:?} is {:?}'s {:?} child ",
                    (*node.0),
                    *node.parent().0,
                    direct
                );
            }
        }
        self.tree_print(node.left(), -1);
        self.tree_print(node.right(), 1);
    }

    pub fn print_tree(&self) {
        if self.root.is_null() {
            println!("This is a empty tree");
            return;
        }
        println!("This tree size = {:?}, begin:-------------", self.len());
        self.tree_print(self.root, 0);
        println!("end--------------------------");
    }
}

/// all key be same, but it has multi key, if has multi key, it perhaps no correct
impl<K, V> PartialEq for RBTree<K, V>
where
    K: Eq + Ord,
    V: PartialEq,
{
    fn eq(&self, other: &RBTree<K, V>) -> bool {
        if self.len() != other.len() {
            return false;
        }

        self.iter()
            .all(|(key, value)| other.get(key).map_or(false, |v| *value == *v))
    }
}

impl<K, V> Eq for RBTree<K, V>
where
    K: Eq + Ord,
    V: Eq,
{
}

impl<'a, K, V> Index<&'a K> for RBTree<K, V>
where
    K: Ord,
{
    type Output = V;

    fn index(&self, index: &K) -> &V {
        self.get(index).expect("no entry found for key")
    }
}

impl<K: Ord, V> FromIterator<(K, V)> for RBTree<K, V> {
    fn from_iter<T: IntoIterator<Item = (K, V)>>(iter: T) -> RBTree<K, V> {
        let mut tree = RBTree::new();
        tree.extend(iter);
        tree
    }
}

impl<K: Ord, V> Extend<(K, V)> for RBTree<K, V> {
    fn extend<T: IntoIterator<Item = (K, V)>>(&mut self, iter: T) {
        let iter = iter.into_iter();
        for (k, v) in iter {
            self.insert(k, v);
        }
    }
}

pub struct Keys<'a, K: Ord + 'a, V: 'a> {
    inner: Iter<'a, K, V>,
}

impl<'a, K: Ord, V> Clone for Keys<'a, K, V> {
    fn clone(&self) -> Keys<'a, K, V> {
        Keys {
            inner: self.inner.clone(),
        }
    }
}

impl<'a, K: Ord + Debug, V> fmt::Debug for Keys<'a, K, V> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_list().entries(self.clone()).finish()
    }
}

impl<'a, K: Ord, V> Iterator for Keys<'a, K, V> {
    type Item = &'a K;

    fn next(&mut self) -> Option<(&'a K)> {
        self.inner.next().map(|(k, _)| k)
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        self.inner.size_hint()
    }
}

pub struct Values<'a, K: 'a + Ord, V: 'a> {
    inner: Iter<'a, K, V>,
}

impl<'a, K: Ord, V> Clone for Values<'a, K, V> {
    fn clone(&self) -> Values<'a, K, V> {
        Values {
            inner: self.inner.clone(),
        }
    }
}

impl<'a, K: Ord + Debug, V: Debug> fmt::Debug for Values<'a, K, V> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_list().entries(self.clone()).finish()
    }
}

impl<'a, K: Ord, V> Iterator for Values<'a, K, V> {
    type Item = &'a V;

    fn next(&mut self) -> Option<(&'a V)> {
        self.inner.next().map(|(_, v)| v)
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        self.inner.size_hint()
    }
}

pub struct ValuesMut<'a, K: 'a + Ord, V: 'a> {
    inner: IterMut<'a, K, V>,
}

impl<'a, K: Ord, V> Clone for ValuesMut<'a, K, V> {
    fn clone(&self) -> ValuesMut<'a, K, V> {
        ValuesMut {
            inner: self.inner.clone(),
        }
    }
}

impl<'a, K: Ord + Debug, V: Debug> fmt::Debug for ValuesMut<'a, K, V> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_list().entries(self.clone()).finish()
    }
}

impl<'a, K: Ord, V> Iterator for ValuesMut<'a, K, V> {
    type Item = &'a mut V;

    fn next(&mut self) -> Option<(&'a mut V)> {
        self.inner.next().map(|(_, v)| v)
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        self.inner.size_hint()
    }
}

pub struct IntoIter<K: Ord, V> {
    head: NodePtr<K, V>,
    tail: NodePtr<K, V>,
    len: usize,
}

impl<K: Ord, V> Drop for IntoIter<K, V> {
    fn drop(&mut self) {
        for (_, _) in self {}
    }
}

impl<K: Ord, V> Iterator for IntoIter<K, V> {
    type Item = (K, V);

    fn next(&mut self) -> Option<(K, V)> {
        if self.len == 0 {
            return None;
        }

        if self.head.is_null() {
            return None;
        }

        let next = self.head.next();
        let obj = unsafe { Box::from_raw(self.head.0) };
        let (k, v) = obj.pair();
        self.head = next;
        self.len -= 1;
        Some((k, v))
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        (self.len, Some(self.len))
    }
}

impl<K: Ord, V> DoubleEndedIterator for IntoIter<K, V> {
    fn next_back(&mut self) -> Option<(K, V)> {
        if self.len == 0 {
            return None;
        }

        if self.tail.is_null() {
            return None;
        }

        let prev = self.tail.prev();
        let obj = unsafe { Box::from_raw(self.tail.0) };
        let (k, v) = obj.pair();
        self.tail = prev;
        self.len -= 1;
        Some((k, v))
    }
}

pub struct Iter<'a, K: Ord + 'a, V: 'a> {
    head: NodePtr<K, V>,
    tail: NodePtr<K, V>,
    len: usize,
    _marker: marker::PhantomData<&'a ()>,
}

impl<'a, K: Ord + 'a, V: 'a> Clone for Iter<'a, K, V> {
    fn clone(&self) -> Iter<'a, K, V> {
        Iter {
            head: self.head,
            tail: self.tail,
            len: self.len,
            _marker: self._marker,
        }
    }
}

impl<'a, K: Ord + 'a, V: 'a> Iterator for Iter<'a, K, V> {
    type Item = (&'a K, &'a V);

    fn next(&mut self) -> Option<(&'a K, &'a V)> {
        if self.len == 0 {
            return None;
        }

        if self.head.is_null() {
            return None;
        }

        let (k, v) = unsafe { (&(*self.head.0).key, &(*self.head.0).value) };
        self.head = self.head.next();
        self.len -= 1;
        Some((k, v))
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        (self.len, Some(self.len))
    }
}

impl<'a, K: Ord + 'a, V: 'a> DoubleEndedIterator for Iter<'a, K, V> {
    fn next_back(&mut self) -> Option<(&'a K, &'a V)> {
        if self.len == 0 {
            return None;
        }

        if self.tail == self.head {
            return None;
        }

        let (k, v) = unsafe { (&(*self.tail.0).key, &(*self.tail.0).value) };
        self.tail = self.tail.prev();
        self.len -= 1;
        Some((k, v))
    }
}

pub struct IterMut<'a, K: Ord + 'a, V: 'a> {
    head: NodePtr<K, V>,
    tail: NodePtr<K, V>,
    len: usize,
    _marker: marker::PhantomData<&'a ()>,
}

impl<'a, K: Ord + 'a, V: 'a> Clone for IterMut<'a, K, V> {
    fn clone(&self) -> IterMut<'a, K, V> {
        IterMut {
            head: self.head,
            tail: self.tail,
            len: self.len,
            _marker: self._marker,
        }
    }
}

impl<'a, K: Ord + 'a, V: 'a> Iterator for IterMut<'a, K, V> {
    type Item = (&'a K, &'a mut V);

    fn next(&mut self) -> Option<(&'a K, &'a mut V)> {
        if self.len == 0 {
            return None;
        }

        if self.head.is_null() {
            return None;
        }

        let (k, v) = unsafe { (&(*self.head.0).key, &mut (*self.head.0).value) };
        self.head = self.head.next();
        self.len -= 1;
        Some((k, v))
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        (self.len, Some(self.len))
    }
}

impl<'a, K: Ord + 'a, V: 'a> DoubleEndedIterator for IterMut<'a, K, V> {
    fn next_back(&mut self) -> Option<(&'a K, &'a mut V)> {
        if self.len == 0 {
            return None;
        }

        if self.tail == self.head {
            return None;
        }

        let (k, v) = unsafe { (&(*self.tail.0).key, &mut (*self.tail.0).value) };
        self.tail = self.tail.prev();
        self.len -= 1;
        Some((k, v))
    }
}

impl<K: Ord, V> IntoIterator for RBTree<K, V> {
    type Item = (K, V);
    type IntoIter = IntoIter<K, V>;

    fn into_iter(mut self) -> IntoIter<K, V> {
        let iter = if self.root.is_null() {
            IntoIter {
                head: NodePtr::null(),
                tail: NodePtr::null(),
                len: self.len,
            }
        } else {
            IntoIter {
                head: self.first_child(),
                tail: self.last_child(),
                len: self.len,
            }
        };
        self.fast_clear();
        iter
    }
}

impl<K: Ord, V> RBTree<K, V> {
    pub fn new() -> RBTree<K, V> {
        RBTree {
            root: NodePtr::null(),
            len: 0,
        }
    }

    pub fn len(&self) -> usize {
        self.len
    }

    pub fn is_empty(&self) -> bool {
        self.root.is_null()
    }

    unsafe fn left_rotate(&mut self, mut node: NodePtr<K, V>) {
        println!("---> rbtree left rotate");
        let mut right = node.right();
        let mut rleft = right.left();
        node.set_right(rleft);
        if !rleft.is_null() {
            rleft.set_parent(node);
        }

        right.set_parent(node.parent());
        if node == self.root {
            self.root = right;
        } else if node == node.parent().left() {
            node.parent().set_left(right);
        } else {
            node.parent().set_right(right);
        }
        right.set_left(node);
        node.set_parent(right);
    }

    unsafe fn right_rotate(&mut self, mut node: NodePtr<K, V>) {
        println!("---> rbtree right rotate");
        let mut left = node.left();
        let mut lright = left.right();
        node.set_left(lright);
        if !lright.is_null() {
            lright.set_parent(node);
        }

        left.set_parent(node.parent());
        if node == self.root {
            self.root = left;
        } else if node == node.parent().right() {
            node.parent().set_right(left);
        } else {
            node.parent().set_left(left);
        }
        left.set_right(node);
        node.set_parent(left);
    }

    unsafe fn insert_fixup(&mut self, mut node: NodePtr<K, V>) {
        while node != self.root {
            let mut parent = node.parent();
            if parent.is_black_color() {
                break;
            }

            let mut uncle = node.uncle();
            let mut grand_parent = node.grand_parent();
            if !uncle.is_null() && uncle.is_red_color() {
                parent.set_black_color();
                uncle.set_black_color();
                grand_parent.set_red_color();
                node = grand_parent;
            } else {
                if node == parent.right() && parent == grand_parent.left() {
                    node = parent;
                    self.left_rotate(parent);
                } else if node == parent.left() && parent == grand_parent.right() {
                    node = parent;
                    self.right_rotate(parent);
                }
                parent = node.parent();
                parent.set_black_color();
                grand_parent.set_red_color();
                if node == parent.left() {
                    self.right_rotate(grand_parent);
                } else {
                    self.left_rotate(grand_parent);
                }
                break;
            }
        }
        self.root.set_black_color();
    }

    pub fn insert(&mut self, k: K, v: V) -> Option<V> {
        let mut y = NodePtr::null();
        let mut x = self.root;

        while !x.is_null() {
            y = x;
            match k.cmp(x.get_key()) {
                Ordering::Less => {
                    x = x.left();
                }
                Ordering::Equal => unsafe {
                    return Some(mem::replace(&mut (*x.0).value, v));
                },
                Ordering::Greater => {
                    x = x.right();
                }
            };
        }

        self.len += 1;
        let mut node = NodePtr::new(k, v);
        node.set_parent(y);

        if y.is_null() {
            self.root = node;
        } else {
            match node.cmp(&&mut y) {
                Ordering::Less => {
                    y.set_left(node);
                }
                _ => {
                    y.set_right(node);
                }
            };
        }

        node.set_red_color();
        unsafe {
            self.insert_fixup(node);
        }
        None
    }

    pub fn find_node(&self, k: &K) -> NodePtr<K, V> {
        let mut current = self.root;
        unsafe {
            loop {
                if current.is_null() {
                    break;
                }
                let next = match k.cmp(&(*current.0).key) {
                    Ordering::Less => (*current.0).left,
                    Ordering::Greater => (*current.0).right,
                    Ordering::Equal => return current,
                };
                current = next;
            }
        }
        NodePtr::null()
    }

    pub fn find_less_equal(&self, k: &K) -> (NodePtr<K, V>, bool) {
        let mut less = NodePtr::null();
        let mut current = self.root;
        unsafe {
            loop {
                if current.is_null() {
                    break;
                }
                let next = match k.cmp(&(*current.0).key) {
                    Ordering::Less => (*current.0).left,
                    Ordering::Greater => {
                        less = current;
                        (*current.0).right
                    }
                    Ordering::Equal => return (current, true),
                };
                current = next;
            }
        }
        (less, false)
    }

    fn first_child(&self) -> NodePtr<K, V> {
        if self.root.is_null() {
            NodePtr::null()
        } else {
            let mut temp = self.root;
            while !temp.left().is_null() {
                temp = temp.left();
            }
            temp
        }
    }

    fn last_child(&self) -> NodePtr<K, V> {
        if self.root.is_null() {
            NodePtr::null()
        } else {
            let mut temp = self.root;
            while !temp.right().is_null() {
                temp = temp.right();
            }
            temp
        }
    }

    pub fn get_first(&self) -> Option<(&K, &V)> {
        let first = self.first_child();
        if first.is_null() {
            return None;
        }
        unsafe { Some((&(*first.0).key, &(*first.0).value)) }
    }

    pub fn get_last(&self) -> Option<(&K, &V)> {
        let last = self.last_child();
        if last.is_null() {
            return None;
        }
        unsafe { Some((&(*last.0).key, &(*last.0).value)) }
    }

    pub fn pop_first(&mut self) -> Option<(K, V)> {
        let first = self.first_child();
        if first.is_null() {
            return None;
        }
        unsafe { Some(self.delete(first)) }
    }

    pub fn pop_last(&mut self) -> Option<(K, V)> {
        let last = self.last_child();
        if last.is_null() {
            return None;
        }
        unsafe { Some(self.delete(last)) }
    }

    pub fn get_first_mut(&mut self) -> Option<(&K, &mut V)> {
        let first = self.first_child();
        if first.is_null() {
            return None;
        }
        unsafe { Some((&(*first.0).key, &mut (*first.0).value)) }
    }

    pub fn get_last_mut(&mut self) -> Option<(&K, &mut V)> {
        let last = self.last_child();
        if last.is_null() {
            return None;
        }
        unsafe { Some((&(*last.0).key, &mut (*last.0).value)) }
    }

    pub fn get(&self, k: &K) -> Option<&V> {
        let node = self.find_node(k);
        if node.is_null() {
            return None;
        }

        unsafe { Some(&(*node.0).value) }
    }

    pub fn get_mut(&mut self, k: &K) -> Option<&mut V> {
        let node = self.find_node(k);
        if node.is_null() {
            return None;
        }

        unsafe { Some(&mut (*node.0).value) }
    }

    pub fn contains_key(&self, k: &K) -> bool {
        let node = self.find_node(k);
        if node.is_null() {
            return false;
        }
        true
    }

    fn clear_recurse(&mut self, current: NodePtr<K, V>) {
        if !current.is_null() {
            unsafe {
                self.clear_recurse(current.left());
                self.clear_recurse(current.right());
                Box::from_raw(current.0);
            }
        }
    }

    pub fn clear(&mut self) {
        let root = self.root;
        self.root = NodePtr::null();
        self.clear_recurse(root);
    }

    fn fast_clear(&mut self) {
        self.root = NodePtr::null();
    }

    pub fn remove(&mut self, k: &K) -> Option<V> {
        let node = self.find_node(k);
        if node.is_null() {
            return None;
        }
        unsafe { Some(self.delete(node).1) }
    }

    unsafe fn delete_fixup(&mut self, mut node: NodePtr<K, V>, mut parent: NodePtr<K, V>) {
        while node != self.root && node.is_black_color() {
            let mut sibling = NodePtr::sibling(parent, node);
            let is_right_sibling = parent.left() == node;
            if sibling.is_red_color() {
                sibling.set_black_color();
                parent.set_red_color();
                if is_right_sibling {
                    self.left_rotate(parent);
                    sibling = parent.right();
                } else {
                    self.right_rotate(parent);
                    sibling = parent.left();
                }
            }

            let mut sibleft = sibling.left();
            let mut sibright = sibling.right();
            if sibleft.is_black_color() && sibright.is_black_color() {
                sibling.set_red_color();
                node = parent;
                parent = node.parent();
            } else {
                if is_right_sibling {
                    if sibright.is_black_color() {
                        sibleft.set_black_color();
                        sibling.set_red_color();
                        self.right_rotate(sibling);
                        sibling = parent.right();
                    }
                } else if sibleft.is_black_color() {
                    sibright.set_black_color();
                    sibling.set_red_color();
                    self.left_rotate(sibling);
                    sibling = parent.left();
                }
                sibling.set_color(parent.get_color());
                parent.set_black_color();
                if is_right_sibling {
                    sibling.right().set_black_color();
                    self.left_rotate(parent);
                } else {
                    sibling.left().set_black_color();
                    self.right_rotate(parent);
                }
                node = self.root;
                break;
            }
        }
        node.set_black_color()
    }

    unsafe fn delete(&mut self, node: NodePtr<K, V>) -> (K, V) {
        let mut child;
        let mut parent;
        let color;

        self.len -= 1;
        if !node.left().is_null() && !node.right().is_null() {
            let mut replace = node.next();
            if node.parent().is_null() {
                self.root = replace;
            } else if node.parent().left() == node {
                node.parent().set_left(replace);
            } else {
                node.parent().set_right(replace);
            }

            child = replace.right();
            parent = replace.parent();
            color = replace.get_color();
            if parent == node {
                parent = replace;
            } else {
                if !child.is_null() {
                    child.set_parent(parent);
                }
                parent.set_left(child);
                replace.set_right(node.right());
                node.right().set_parent(replace);
            }

            replace.set_parent(node.parent());
            replace.set_color(node.get_color());
            replace.set_left(node.left());
            node.left().set_parent(replace);

            if color == Color::Black {
                self.delete_fixup(child, parent);
            }

            return Box::from_raw(node.0).pair();
        }

        if !node.left().is_null() {
            child = node.left();
        } else {
            child = node.right();
        }
        if !child.is_null() {
            child.set_parent(node.parent());
        }

        if node.parent().is_null() {
            self.root = child
        } else if node.parent().left() == node {
            node.parent().set_left(child);
        } else {
            node.parent().set_right(child);
        }

        if node.is_black_color() {
            self.delete_fixup(child, node.parent());
        }

        Box::from_raw(node.0).pair()
    }

    pub fn keys(&self) -> Keys<K, V> {
        Keys { inner: self.iter() }
    }

    pub fn values(&self) -> Values<K, V> {
        Values { inner: self.iter() }
    }

    pub fn values_mut(&mut self) -> ValuesMut<K, V> {
        ValuesMut {
            inner: self.iter_mut(),
        }
    }

    pub fn iter(&self) -> Iter<K, V> {
        Iter {
            head: self.first_child(),
            tail: self.last_child(),
            len: self.len,
            _marker: marker::PhantomData,
        }
    }

    pub fn iter_mut(&mut self) -> IterMut<K, V> {
        IterMut {
            head: self.first_child(),
            tail: self.last_child(),
            len: self.len,
            _marker: marker::PhantomData,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::RBTree;
    #[test]
    fn test_insert() {
        let mut m = RBTree::new();
        assert_eq!(m.len(), 0);
        m.insert(1, 2);
        assert_eq!(m.len(), 1);
        m.insert(2, 4);
        assert_eq!(m.len(), 2);
        m.insert(2, 6);
        assert_eq!(m.len(), 2);
        assert_eq!(*m.get(&1).unwrap(), 2);
        assert_eq!(*m.get(&2).unwrap(), 6);
    }

    #[test]
    fn test_clone() {
        let mut m = RBTree::new();
        assert_eq!(m.len(), 0);
        m.insert(1, 2);
        assert_eq!(m.len(), 1);
        m.insert(2, 4);
        assert_eq!(m.len(), 2);
        let m2 = m.clone();
        m.clear();
        assert_eq!(*m2.get(&1).unwrap(), 2);
        assert_eq!(*m2.get(&2).unwrap(), 4);
        assert_eq!(m2.len(), 2);
    }

    #[test]
    fn test_empty_remove() {
        let mut m: RBTree<isize, bool> = RBTree::new();
        assert_eq!(m.remove(&0), None);
    }

    #[test]
    fn test_empty_iter() {
        let mut m: RBTree<isize, bool> = RBTree::new();
        assert_eq!(m.iter().next(), None);
        assert_eq!(m.iter_mut().next(), None);
        assert_eq!(m.len(), 0);
        assert!(m.is_empty());
        assert_eq!(m.into_iter().next(), None);
    }

    #[test]
    fn test_lots_of_insertions() {
        let mut m = RBTree::new();

        for _ in 0..10 {
            assert!(m.is_empty());

            for i in 1..101 {
                m.insert(i, i);

                for j in 1..i + 1 {
                    let r = m.get(&j);
                    assert_eq!(r, Some(&j));
                }

                for j in i + 1..101 {
                    let r = m.get(&j);
                    assert_eq!(r, None);
                }
            }

            for i in 101..201 {
                assert!(!m.contains_key(&i));
            }

            // remove forwards
            for i in 1..101 {
                assert!(m.remove(&i).is_some());

                for j in 1..i + 1 {
                    assert!(!m.contains_key(&j));
                }

                for j in i + 1..101 {
                    assert!(m.contains_key(&j));
                }
            }

            for i in 1..101 {
                assert!(!m.contains_key(&i));
            }

            for i in 1..101 {
                m.insert(i, i);
            }

            // remove backwards
            for i in (1..101).rev() {
                assert!(m.remove(&i).is_some());

                for j in i..101 {
                    assert!(!m.contains_key(&j));
                }

                for j in 1..i {
                    assert!(m.contains_key(&j));
                }
            }
        }
    }

    #[test]
    fn test_find_mut() {
        let mut m = RBTree::new();
        assert_eq!(m.get_mut(&5), None);
        m.insert(1, 12);
        m.insert(2, 8);
        m.insert(5, 14);
        let new = 100;
        match m.get_mut(&5) {
            None => panic!(),
            Some(x) => *x = new,
        }
        assert_eq!(m.get(&5), Some(&new));
    }

    #[test]
    fn test_find_less_eqaul() {
        let mut m = RBTree::new();
        let (node, found) = m.find_less_equal(&5);
        assert!(node.is_null());
        assert_eq!(found, false);
        m.insert(1, 12);
        m.insert(2, 8);
        m.insert(5, 14);
        let (node, found) = m.find_less_equal(&3);
        assert_eq!(*node.get_key(), 2);
        assert_eq!(found, false);

        let (node, found) = m.find_less_equal(&5);
        assert_eq!(*node.get_key(), 5);
        assert_eq!(found, true);
    }

    #[test]
    fn test_remove() {
        let mut m = RBTree::new();
        m.insert(1, 2);
        assert_eq!(*m.get(&1).unwrap(), 2);
        m.insert(5, 3);
        assert_eq!(*m.get(&5).unwrap(), 3);
        m.insert(9, 4);
        assert_eq!(*m.get(&1).unwrap(), 2);
        assert_eq!(*m.get(&5).unwrap(), 3);
        assert_eq!(*m.get(&9).unwrap(), 4);
        assert_eq!(m.remove(&1).unwrap(), 2);
        assert_eq!(m.remove(&5).unwrap(), 3);
        assert_eq!(m.remove(&9).unwrap(), 4);
        assert_eq!(m.remove(&3), None);
        assert_eq!(m.remove(&9), None);
        assert_eq!(m.len(), 0);
    }

    #[test]
    fn test_is_empty() {
        let mut m = RBTree::new();
        m.insert(1, 2);
        assert!(!m.is_empty());
        assert!(m.remove(&1).is_some());
        assert!(m.is_empty());
    }

    #[test]
    fn test_pop() {
        let mut m = RBTree::new();
        m.insert(2, 4);
        m.insert(1, 2);
        m.insert(3, 6);
        assert_eq!(m.len(), 3);
        assert_eq!(m.pop_first(), Some((1, 2)));
        assert_eq!(m.len(), 2);
        assert_eq!(m.pop_last(), Some((3, 6)));
        assert_eq!(m.len(), 1);
        assert_eq!(m.get_first(), Some((&2, &4)));
        assert_eq!(m.get_last(), Some((&2, &4)));
    }

    #[test]
    fn test_iterate() {
        let mut m = RBTree::new();
        for i in 0..32 {
            m.insert(i, i * 2);
        }
        assert_eq!(m.len(), 32);

        let mut observed: u32 = 0;

        for (k, v) in m.iter() {
            assert_eq!(*v, *k * 2);
            observed |= 1 << *k;
        }
        assert_eq!(observed, 0xFFFF_FFFF);
    }

    #[test]
    fn test_keys() {
        let vec = vec![(1, 'a'), (2, 'b'), (3, 'c')];
        let map: RBTree<_, _> = vec.into_iter().collect();
        let keys: Vec<_> = map.keys().cloned().collect();
        assert_eq!(keys.len(), 3);
        assert!(keys.contains(&1));
        assert!(keys.contains(&2));
        assert!(keys.contains(&3));
    }

    #[test]
    fn test_values() {
        let vec = vec![(1, 'a'), (2, 'b'), (3, 'c')];
        let map: RBTree<_, _> = vec.into_iter().collect();
        let values: Vec<_> = map.values().cloned().collect();
        assert_eq!(values.len(), 3);
        assert!(values.contains(&'a'));
        assert!(values.contains(&'b'));
        assert!(values.contains(&'c'));
    }

    #[test]
    fn test_values_mut() {
        let vec = vec![(1, 1), (2, 2), (3, 3)];
        let mut map: RBTree<_, _> = vec.into_iter().collect();
        for value in map.values_mut() {
            *value = (*value) * 2
        }
        let values: Vec<_> = map.values().cloned().collect();
        assert_eq!(values.len(), 3);
        assert!(values.contains(&2));
        assert!(values.contains(&4));
        assert!(values.contains(&6));
    }

    #[test]
    fn test_find() {
        let mut m = RBTree::new();
        assert!(m.get(&1).is_none());
        m.insert(1, 2);
        match m.get(&1) {
            None => panic!(),
            Some(v) => assert_eq!(*v, 2),
        }
    }

    #[test]
    fn test_eq() {
        let mut m1 = RBTree::new();
        m1.insert(1, 2);
        m1.insert(2, 3);
        m1.insert(3, 4);

        let mut m2 = RBTree::new();
        m2.insert(1, 2);
        m2.insert(2, 3);

        assert!(m1 != m2);

        m2.insert(3, 4);

        assert_eq!(m1, m2);
    }

    #[test]
    fn test_show() {
        let mut map = RBTree::new();
        let empty: RBTree<i32, i32> = RBTree::new();

        map.insert(1, 2);
        map.insert(3, 4);

        let map_str = format!("{:?}", map);

        assert!(map_str == "{1: 2, 3: 4}" || map_str == "{3: 4, 1: 2}");
        assert_eq!(format!("{:?}", empty), "{}");
    }

    #[test]
    fn test_from_iter() {
        let xs = [(1, 1), (2, 2), (3, 3), (4, 4), (5, 5), (6, 6)];

        let map: RBTree<_, _> = xs.iter().cloned().collect();

        for &(k, v) in &xs {
            assert_eq!(map.get(&k), Some(&v));
        }
    }

    #[test]
    fn test_size_hint() {
        let xs = [(1, 1), (2, 2), (3, 3), (4, 4), (5, 5), (6, 6)];

        let map: RBTree<_, _> = xs.iter().cloned().collect();

        let mut iter = map.iter();

        for _ in iter.by_ref().take(3) {}

        assert_eq!(iter.size_hint(), (3, Some(3)));
    }

    #[test]
    fn test_iter_len() {
        let xs = [(1, 1), (2, 2), (3, 3), (4, 4), (5, 5), (6, 6)];

        let map: RBTree<_, _> = xs.iter().cloned().collect();

        let mut iter = map.iter();

        for _ in iter.by_ref().take(3) {}

        assert_eq!(iter.count(), 3);
    }

    #[test]
    fn test_mut_size_hint() {
        let xs = [(1, 1), (2, 2), (3, 3), (4, 4), (5, 5), (6, 6)];

        let mut map: RBTree<_, _> = xs.iter().cloned().collect();

        let mut iter = map.iter_mut();

        for _ in iter.by_ref().take(3) {}

        assert_eq!(iter.size_hint(), (3, Some(3)));
    }

    #[test]
    fn test_iter_mut_len() {
        let xs = [(1, 1), (2, 2), (3, 3), (4, 4), (5, 5), (6, 6)];

        let mut map: RBTree<_, _> = xs.iter().cloned().collect();

        let mut iter = map.iter_mut();

        for _ in iter.by_ref().take(3) {}

        assert_eq!(iter.count(), 3);
    }

    #[test]
    fn test_index() {
        let mut map = RBTree::new();

        map.insert(1, 2);
        map.insert(2, 1);
        map.insert(3, 4);

        assert_eq!(map[&2], 1);
    }

    #[test]
    #[should_panic]
    fn test_index_nonexistent() {
        let mut map = RBTree::new();

        map.insert(1, 2);
        map.insert(2, 1);
        map.insert(3, 4);

        map[&4];
    }

    #[test]
    fn test_extend_iter() {
        let mut a = RBTree::new();
        a.insert(1, "one");
        let mut b = RBTree::new();
        b.insert(2, "two");
        b.insert(3, "three");

        a.extend(b.into_iter());

        assert_eq!(a.len(), 3);
        assert_eq!(a[&1], "one");
        assert_eq!(a[&2], "two");
        assert_eq!(a[&3], "three");
    }

    #[test]
    fn test_tree_print() {
        let mut a = RBTree::new();
        a.insert("a", 1);
        a.insert("b", 2);
        a.insert("c", 3);
        a.print_tree();

        a.remove(&"a");
        a.print_tree();
    }
}
