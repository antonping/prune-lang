use rand::distr::Distribution;
use rand::distr::weighted::WeightedIndex;
use rand::{RngExt, rngs};
use std::fmt;

pub type IdxPair = (usize, usize);

#[derive(Debug, Clone)]
pub struct PathTrie {
    children: Vec<(IdxPair, PathTrie)>,
    weight: usize,
}

impl fmt::Display for PathTrie {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "(root)")?;
        self.fmt_with_prefix(f, "")
    }
}

impl PathTrie {
    fn fmt_with_prefix(&self, f: &mut fmt::Formatter<'_>, prefix: &str) -> fmt::Result {
        write!(f, "{{{}}}", self.weight)?;

        if !self.children.is_empty() {
            for (i, (pair, child)) in self.children.iter().enumerate() {
                if i == self.children.len() - 1 {
                    write!(f, "\n{prefix}└─ {:?}", pair)?;
                    child.fmt_with_prefix(f, &format!("{prefix}  "))?;
                } else {
                    write!(f, "\n{prefix}├─ {:?}", pair)?;
                    child.fmt_with_prefix(f, &format!("{prefix}│  "))?;
                }
            }
            Ok(())
        } else {
            write!(f, "\n{prefix}└─ ?")
        }
    }
}

impl PathTrie {
    pub fn new() -> Self {
        PathTrie {
            children: Vec::new(),
            weight: 0,
        }
    }

    fn find_idx(&self, pair: &IdxPair) -> Option<usize> {
        self.children.iter().enumerate().find_map(
            |(idx, (k, _v))| {
                if *k == *pair { Some(idx) } else { None }
            },
        )
    }

    pub fn insert(&mut self, path: &[IdxPair]) {
        let mut temp_self = self;
        let mut temp_path = path;

        while let [pair, ..] = temp_path {
            temp_path = &temp_path[1..];
            if let Some(idx) = temp_self.find_idx(pair) {
                temp_self = &mut temp_self.children[idx].1;
            } else {
                temp_self.children.push((*pair, PathTrie::new()));
                temp_self = &mut temp_self.children.last_mut().unwrap().1;
            }
        }
    }

    pub fn choose_random_path(&self, rng: &mut rngs::ThreadRng) -> Vec<IdxPair> {
        let mut path_res = Vec::new();
        let mut temp_self = self;
        loop {
            if !temp_self.children.is_empty() {
                let n = rng.random_range(0..temp_self.children.len());
                let (pair, next_node) = &temp_self.children[n];
                path_res.push(*pair);
                temp_self = next_node;
            } else {
                return path_res;
            }
        }
    }

    pub fn choose_random_path_weighted(&self, rng: &mut rngs::ThreadRng) -> Vec<IdxPair> {
        let mut path_res = Vec::new();
        let mut temp_self = self;
        loop {
            if !temp_self.children.is_empty() {
                let weights = temp_self.children.iter().map(|(_, child)| child.weight);
                let dist = WeightedIndex::new(weights.map(|w| 1.0 / (w as f32 + 0.1))).unwrap();
                let n = dist.sample(rng);
                let (pair, next_node) = &temp_self.children[n];
                path_res.push(*pair);
                temp_self = next_node;
            } else {
                return path_res;
            }
        }
    }

    pub fn choose_shortest_path(&self) -> Vec<IdxPair> {
        let mut min_depth = usize::MAX;
        let mut min_path = Vec::new();
        let mut cur_depth = 0;
        let mut cur_path = Vec::new();
        self.choose_shortest_path_help(
            &mut min_depth,
            &mut min_path,
            &mut cur_depth,
            &mut cur_path,
        );
        min_path
    }

    fn choose_shortest_path_help(
        &self,
        min_depth: &mut usize,
        min_path: &mut Vec<IdxPair>,
        cur_depth: &mut usize,
        cur_path: &mut Vec<IdxPair>,
    ) {
        if *cur_depth >= *min_depth {
            return;
        }
        if !self.children.is_empty() {
            for (pair, child) in &self.children {
                *cur_depth += 1;
                cur_path.push(*pair);
                child.choose_shortest_path_help(min_depth, min_path, cur_depth, cur_path);
                *cur_depth -= 1;
                cur_path.pop();
            }
        } else {
            *min_depth = *cur_depth;
            *min_path = cur_path.clone();
        }
    }

    pub fn expand_trie(&mut self, path: &[IdxPair], subtries: PathTrie) {
        let mut temp_self = self;
        let mut temp_path = path;
        while let [pair, ..] = temp_path {
            temp_path = &temp_path[1..];
            let idx = temp_self.find_idx(pair).unwrap();
            temp_self = &mut temp_self.children[idx].1;
        }
        *temp_self = subtries;
    }

    pub fn remove_trie(&mut self, path: &[IdxPair]) -> bool {
        if let [pair, ..] = path {
            let idx = self.find_idx(pair).unwrap();
            if self.children[idx].1.remove_trie(&path[1..]) {
                self.children.remove(idx);
                self.children.is_empty()
            } else {
                false
            }
        } else {
            assert!(self.children.is_empty());
            true
        }
    }

    pub fn incr_weight(&mut self, path: &[IdxPair]) {
        let mut temp_self = self;
        let mut temp_path = path;
        while let [pair, ..] = temp_path {
            temp_self.weight += 1;
            temp_path = &temp_path[1..];
            let idx = temp_self.find_idx(pair).unwrap();
            temp_self = &mut temp_self.children[idx].1;
        }
        assert!(temp_self.children.is_empty());
        temp_self.weight += 1;
    }
}
