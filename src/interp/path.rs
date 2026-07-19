use itertools::Itertools;
use rand::{RngExt, rngs};
use std::fmt;

#[derive(Debug, Clone)]
pub enum PathTrie {
    Node(usize, Vec<(usize, PathTrie)>),
    Unexpanded,
}

impl fmt::Display for PathTrie {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.fmt_with_prefix(f, "")
    }
}

impl PathTrie {
    fn fmt_with_prefix(&self, f: &mut fmt::Formatter<'_>, prefix: &str) -> fmt::Result {
        match self {
            PathTrie::Unexpanded => {
                write!(f, "\n{prefix}└─ ?")
            }
            PathTrie::Node(call_idx, children) => {
                for (i, (rule_idx, child)) in children.iter().enumerate() {
                    let is_last = i == children.len() - 1;
                    let branch = if is_last { "└─" } else { "├─" };
                    let child_prefix = if is_last { "  " } else { "│ " };
                    write!(f, "\n{prefix}{branch}({call_idx},{rule_idx})")?;
                    child.fmt_with_prefix(f, &format!("{prefix}{child_prefix}"))?;
                }
                Ok(())
            }
        }
    }
}

impl PathTrie {
    /// Creates a new PathTrie node (non-end node)
    pub fn new() -> Self {
        PathTrie::Unexpanded
    }

    /// Checks if the node is empty (no children and not an end)
    pub fn is_empty(&self) -> bool {
        matches!(self, PathTrie::Node(_, children) if children.is_empty())
    }

    /// Returns whether this node is an end node
    pub fn is_unexpanded(&self) -> bool {
        matches!(self, PathTrie::Unexpanded)
    }

    /// Inserts a path into the trie
    pub fn insert(&mut self, path: &[(usize, usize)]) {
        let mut temp_self = self;
        let mut temp_path = path;

        loop {
            if temp_path.is_empty() {
                return;
            }

            match temp_self {
                PathTrie::Node(cur_call, children) => {
                    assert!(!children.is_empty());
                    let (call_idx, rule_idx) = temp_path[0];
                    temp_path = &temp_path[1..];
                    assert_eq!(*cur_call, call_idx);
                    let idx = children
                        .iter()
                        .find_position(|(k, _)| *k == rule_idx)
                        .map(|x| x.0);
                    if let Some(idx) = idx {
                        temp_self = &mut children.get_mut(idx).unwrap().1;
                    } else {
                        children.push((rule_idx, PathTrie::new()));
                        temp_self = &mut children.last_mut().unwrap().1;
                    }
                }
                PathTrie::Unexpanded => {
                    let (call_idx, rule_idx) = temp_path[0];
                    *temp_self = PathTrie::Node(call_idx, vec![(rule_idx, PathTrie::Unexpanded)]);
                }
            };
        }
    }

    // // /// Checks if a path exists in the trie
    // pub fn contains(&self, path: &[u8]) -> bool {
    //     let mut temp_self = self;
    //     let mut temp_path = path;

    //     loop {
    //         match temp_self {
    //             PathTrie::Node(_) if temp_path.is_empty() => return false,
    //             PathTrie::Node(children) => {
    //                 let step = temp_path[0];
    //                 temp_path = &temp_path[1..];
    //                 match children.iter().find(|(k, _)| *k == step) {
    //                     Some((_, child)) => temp_self = child,
    //                     None => return false,
    //                 }
    //             }
    //             PathTrie::Unexpanded => {
    //                 assert!(temp_path.is_empty());
    //                 return true;
    //             }
    //         }
    //     }
    // }

    pub fn random_unexpaned_path(&self, rnd: &mut rngs::ThreadRng) -> Vec<(usize, usize)> {
        let mut path_res = Vec::new();
        let mut temp_self = self;
        loop {
            match temp_self {
                PathTrie::Node(call_idx, children) => {
                    assert!(!children.is_empty());
                    let n = rnd.random_range(0..children.len());
                    let (rule_idx, v) = &children[n];
                    path_res.push((*call_idx, *rule_idx));
                    temp_self = v;
                }
                PathTrie::Unexpanded => return path_res,
            }
        }
    }

    pub fn expand_trie(&mut self, path: &[(usize, usize)], subtrie: PathTrie) {
        let mut temp_self = self;
        let mut temp_path = path;

        loop {
            match temp_self {
                PathTrie::Node(cur_call, children) => {
                    assert!(!children.is_empty());
                    let (call_idx, rule_idx) = temp_path[0];
                    temp_path = &temp_path[1..];
                    assert_eq!(*cur_call, call_idx);
                    match children.iter_mut().find(|(k, _)| *k == rule_idx) {
                        Some((_, child)) => temp_self = child,
                        None => panic!("cannot find the path!"),
                    }
                }
                PathTrie::Unexpanded => {
                    assert!(temp_path.is_empty());
                    break;
                }
            }
        }
        *temp_self = subtrie;
    }

    pub fn remove_trie(&mut self, path: &[(usize, usize)]) -> bool {
        match self {
            PathTrie::Node(cur_call, children) => {
                assert!(!children.is_empty());
                let (call_idx, rule_idx) = path[0];
                let rest_path = &path[1..];
                assert_eq!(*cur_call, call_idx);
                let idx = children
                    .iter()
                    .find_position(|(k, _)| *k == rule_idx)
                    .unwrap()
                    .0;
                if children.get_mut(idx).unwrap().1.remove_trie(rest_path) {
                    children.remove(idx);
                    return children.is_empty();
                } else {
                    return false;
                }
            }
            PathTrie::Unexpanded => {
                assert!(path.is_empty());
                true
            }
        }
    }
}

impl Default for PathTrie {
    fn default() -> Self {
        Self::new()
    }
}

// #[test]
// fn test_path_insert_and_contains() {
//     let mut trie = PathTrie::new();

//     trie.insert(b"hello");
//     trie.insert(b"world");
//     trie.insert(b"hi");

//     assert!(trie.contains(b"hello"));
//     assert!(trie.contains(b"world"));
//     assert!(trie.contains(b"hi"));
//     assert!(!trie.contains(b"hell"));
//     assert!(!trie.contains(b"wor"));
//     assert!(!trie.contains(b""));
// }

// #[test]
// fn test_path_get_existing_key() {
//     let mut trie = PathTrie::new();
//     trie.insert(b"abc");

//     let child = trie.get_step(b'a').unwrap();
//     assert!(!child.is_unexpanded());

//     let grandchild = child.get_step(b'b').unwrap();
//     assert!(!grandchild.is_unexpanded());

//     let leaf = grandchild.get_step(b'c').unwrap();
//     assert!(leaf.is_unexpanded());
// }

// #[test]
// fn test_path_get_path_full() {
//     let mut trie = PathTrie::new();
//     trie.insert(b"hello");

//     let node = trie.get_path(b"hello").unwrap();
//     assert!(node.is_unexpanded());
// }

// #[test]
// fn test_path_insert_shared_prefix() {
//     let mut trie = PathTrie::new();
//     trie.insert(b"abc");
//     trie.insert(b"abd");

//     assert!(trie.contains(b"abc"));
//     assert!(trie.contains(b"abd"));
//     assert!(!trie.contains(b"ab"));

//     let mid = trie.get_path(b"ab").unwrap();
//     let mut keys = mid.next_step().unwrap();
//     keys.sort();
//     assert_eq!(keys, vec![b'c', b'd']);
// }
