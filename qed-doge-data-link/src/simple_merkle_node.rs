use std::cmp::Ordering;
use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct SimpleMerkleNodeKey {
    pub level: u8,
    pub index: u64,
}
impl PartialOrd for SimpleMerkleNodeKey {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        if self.level != other.level {
            self.level.partial_cmp(&other.level)
        }else{
            self.index.partial_cmp(&other.index)
        }
    }
}

impl SimpleMerkleNodeKey {
    pub fn new_root() -> Self {
        Self { level: 0, index: 0 }
    }
    pub fn new(level: u8, index: u64) -> Self {
        Self { level, index }
    }
    pub fn first_leaf_for_height(&self, height: u8) -> Self {
        if height <= self.level {
            self.clone()
        } else {
            let diff = (height - self.level) as u64;
            Self {
                level: height,
                index: (1u64 << diff) * self.index,
            }
        }
    }
    pub fn sibling(&self) -> Self {
        Self {
            level: self.level,
            index: self.index ^ 1,
        }
    }
    pub fn siblings(&self) -> Vec<Self> {
        let mut result = Vec::with_capacity(self.level as usize);
        let mut current = *self;
        for _ in 0..self.level {
            result.push(current.sibling());
            current = current.parent();
        }
        result
    }

    // if self or other are on the same merkle path
    pub fn is_direct_path_related(&self, other: &SimpleMerkleNodeKey) -> bool {
        if other.level == self.level {
            self.index == other.index
        }else if other.level < self.level {
            // opt?: (self.index>>(self.level-other.level)) == other.index
            self.parent_at_level(other.level).index == other.index

        }else{
            other.parent_at_level(self.level).index == self.index
        }
    }
    pub fn parent(&self) -> Self {
        if self.level == 0 {
            return *self;
        }
        Self {
            level: self.level - 1,
            index: self.index >> 1,
        }
    }
    pub fn first_leaf_child(&self, tree_height: u8) -> Self {
        Self {
            level: tree_height,
            index: self.index << (tree_height - self.level),
        }
    }
    pub fn left_child(&self) -> Self {
        Self {
            level: self.level + 1,
            index: self.index << 1,
        }
    }
    pub fn right_child(&self) -> Self {
        Self {
            level: self.level + 1,
            index: (self.index << 1) + 1,
        }
    }
    pub fn is_on_the_right_of(&self, other: &SimpleMerkleNodeKey) -> bool {
        if other.level == self.level {
            self.index > other.index
        } else if other.level < self.level {
            self.parent_at_level(other.level).index > other.index
        } else {
            self.index > other.parent_at_level(self.level).index
        }
    }
    pub fn is_to_the_left_of(&self, other: &SimpleMerkleNodeKey) -> bool {
        if other.level == self.level {
            self.index < other.index
        } else if other.level < self.level {
            self.parent_at_level(other.level).index < other.index
        } else {
            self.index < other.parent_at_level(self.level).index
        }
    }

    pub fn parent_at_level(&self, level: u8) -> Self {
        if level > self.level {
            panic!("given level is not above this node")
        }
        self.n_th_ancestor(self.level - level)
    }
    pub fn n_th_ancestor(&self, levels_above: u8) -> Self {
        if levels_above >= self.level {
            Self::new_root()
        } else {
            Self {
                level: self.level - levels_above,
                index: self.index >> levels_above,
            }
        }
    }
    pub fn find_nearest_common_ancestor(&self, other: &SimpleMerkleNodeKey) -> SimpleMerkleNodeKey {
        let start_level = u8::min(other.level, self.level);
        let mut self_current = self.parent_at_level(start_level);
        let mut other_current = other.parent_at_level(start_level);
        while !other_current.eq(&self_current) {
            self_current = self_current.parent();
            other_current = other_current.parent();
        }
        self_current
    }
}


#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct SimpleMerkleNode<Hash> {
    pub key: SimpleMerkleNodeKey,
    pub value: Hash,
}

impl<Hash> SimpleMerkleNode<Hash> {
    pub fn new_root(value: Hash) -> Self {
        Self {
            key: SimpleMerkleNodeKey::new_root(),
            value,
        }
    }
    pub fn new(level: u8, index: u64, value: Hash) -> Self {
        Self {
            key: SimpleMerkleNodeKey { level, index },
            value,
        }
    }
}
impl<Hash: PartialOrd> PartialOrd for SimpleMerkleNode<Hash> {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        if self.key.level != other.key.level {
            self.key.level.partial_cmp(&other.key.level)
        }else if self.key.index != other.key.index {
            self.key.index.partial_cmp(&other.key.index)
        }else {
            self.value.partial_cmp(&other.value)
        }
    }
}