use std::{collections::HashMap, marker::PhantomData};

use doge_light_client::hash::{merkle::{delta_merkle_proof::DeltaMerkleProofCore, merkle_proof::MerkleProofCore}, traits::{MerkleHasher, ZeroableHash}};

use crate::simple_merkle_node::SimpleMerkleNodeKey;


#[derive(Clone)]
pub struct SimpleMerkleTree<Hasher, Hash: Copy + PartialEq + Default> {
    nodes: HashMap<SimpleMerkleNodeKey, Hash>,
    height: u8,
    zero_value_hashes: Vec<Hash>,
    _hasher: PhantomData<Hasher>,
}

impl<Hasher: MerkleHasher<Hash>, Hash: Copy + PartialEq + Default + ZeroableHash>
    SimpleMerkleTree<Hasher, Hash>
{
    pub fn new(height: u8) -> Self {

        let mut zero_hash = Hash::get_zero_value();
        let mut zero_value_hashes = Vec::with_capacity((height + 1) as usize);
        for _ in 0..(height + 1) {
            zero_value_hashes.push(zero_hash);
            zero_hash = Hasher::two_to_one(&zero_hash, &zero_hash);
        }
        Self {
            nodes: HashMap::new(),
            height,
            zero_value_hashes,
            _hasher: PhantomData::default(),
        }
    }
    pub fn get_height(&self) -> u8 {
        self.height
    }
    pub fn get_max_leaf_index(&self) -> u64 {
        (1u64 << (self.height as u64)) - 1u64
    }
    pub fn set_node_value(&mut self, key: SimpleMerkleNodeKey, value: Hash) {
        if value.eq(&self.zero_value_hashes[(self.height - key.level) as usize]) {
            self.nodes.remove(&key);
        } else {
            self.nodes.insert(key, value);
        }
    }
    pub fn get_node_value(&self, key: &SimpleMerkleNodeKey) -> Hash {
        if self.nodes.contains_key(key) {
            self.nodes[key]
        } else {
            assert!(
                self.height >= key.level,
                "requested node value of invalid key level for this tree"
            );
            self.zero_value_hashes[(self.height - key.level) as usize]
            //Hasher::get_zero_hash((self.height - key.level) as usize)
        }
    }

    pub fn get_root(&self) -> Hash {
        self.get_node_value(&SimpleMerkleNodeKey::new_root())
    }

    pub fn get_leaf_value(&self, index: u64) -> Hash {
        self.get_node_value(&SimpleMerkleNodeKey::new(self.height, index))
    }

    pub fn get_leaf(&self, index: u64) -> MerkleProofCore<Hash> {
        let leaf_key = SimpleMerkleNodeKey::new(self.height, index);
        let value = self.get_leaf_value(index);

        let mut current_sibling = leaf_key.sibling();
        let mut siblings = Vec::with_capacity(self.height as usize);

        while current_sibling.level > 0 {
            siblings.push(self.get_node_value(&current_sibling));
            current_sibling = current_sibling.parent().sibling();
        }

        let root = self.get_root();

        MerkleProofCore {
            index,
            siblings,
            root,
            value,
        }
    }
    pub fn find_first_non_zero_leaf(&self, node: SimpleMerkleNodeKey) -> Option<u64> {
        let value = self.get_node_value(&node);
        let zero_hash = self.zero_value_hashes[(self.height - node.level) as usize];
        if value.eq(&zero_hash) {
            Some(node.first_leaf_child(self.height).index)
        } else {
            if self.height == node.level {
                if value.eq(&zero_hash) {
                    Some(node.index)
                } else {
                    None
                }
            } else {
                match self.find_first_non_zero_leaf(node.left_child()) {
                    Some(ind) => Some(ind),
                    None => self.find_first_non_zero_leaf(node.right_child()),
                }
            }
        }
    }
    pub fn find_next_append_index(&self) -> anyhow::Result<u64> {
        if self
            .get_root()
            .eq(&self.zero_value_hashes[self.height as usize])
        {
            return Ok(0);
        } else if self.height == 0 {
            anyhow::bail!("tree is full");
        } else {
            /*
            let mut cur_node = SimpleMerkleNodeKey::new_root();
            while cur_node.level < self.height {
                let child_zero_hash =
                    Hasher::get_zero_hash((self.height - (cur_node.level + 1)) as usize);
                let left_child = cur_node.left_child();
                let right_child = cur_node.right_child();
                println!("cur_node: {:?}", cur_node);
                if self.get_node_value(&left_child).eq(&child_zero_hash) {
                    return Ok(left_child.first_leaf_child(self.height).index);
                } else if self.get_node_value(&right_child).eq(&child_zero_hash) {
                    cur_node = left_child;
                } else {
                    cur_node = right_child;
                }
            }

            anyhow::bail!("tree is full");*/

            match self.find_first_non_zero_leaf(SimpleMerkleNodeKey::new_root()) {
                Some(ind) => Ok(ind),
                None => anyhow::bail!("tree is full"),
            }
        }
    }
    pub fn rehash_from_node_to_level(&mut self, node: SimpleMerkleNodeKey, root_level: u8) {
        let mut current = node;
        let mut current_value = self.get_node_value(&current);
        while current.level > root_level {
            let parent_key = current.parent();
            let sibling_value = self.get_node_value(&current.sibling());

            let parent_value = if (current.index & 1) == 1 {
                Hasher::two_to_one(&sibling_value, &current_value)
            } else {
                Hasher::two_to_one(&current_value, &sibling_value)
            };
            self.set_node_value(parent_key, parent_value);
            current = parent_key;
            current_value = parent_value;
        }
    }

    pub fn rehash_sub_tree_dmp(
        &mut self,
        sub_tree_height: u8,
        sub_tree_index: u64,
    ) -> DeltaMerkleProofCore<Hash> {
        let sub_tree_root_level = self.height - sub_tree_height;
        let sub_root_node = SimpleMerkleNodeKey::new(sub_tree_root_level, sub_tree_index);
        let old_sub_tree_root = self.get_node_value(&sub_root_node);
        let old_tree_root = self.get_root();

        self.rehash_sub_tree(sub_tree_height, sub_tree_index);

        let new_sub_tree_root = self.get_node_value(&sub_root_node);
        let new_tree_root = self.get_root();

        let siblings = sub_root_node
            .siblings()
            .iter()
            .map(|x| self.get_node_value(x))
            .collect::<Vec<_>>();

        DeltaMerkleProofCore {
            old_root: old_tree_root,
            old_value: old_sub_tree_root,
            new_root: new_tree_root,
            new_value: new_sub_tree_root,
            index: sub_tree_index,
            siblings,
        }
    }
    pub fn rehash_sub_tree(&mut self, sub_tree_height: u8, sub_tree_index: u64) -> Hash {
        if sub_tree_height == 0 {
            return self.get_leaf_value(sub_tree_index);
        } else if sub_tree_height == 1 {
            let left_key = SimpleMerkleNodeKey::new(self.height, sub_tree_index * 2);
            let v = Hasher::two_to_one(
                &self.get_node_value(&left_key),
                &self.get_node_value(&SimpleMerkleNodeKey::new(self.height, sub_tree_index * 2)),
            );
            self.set_node_value(left_key.parent(), v);
            return v;
        }

        let sub_tree_root_level = self.height - sub_tree_height;

        let mut child_base_key = SimpleMerkleNodeKey::new(
            self.height,
            (sub_tree_index) * (1u64 << (sub_tree_height as u64)),
        );

        let mut nodes_at_current_level = 1usize << (sub_tree_height - 1);

        let mut child_values = Vec::with_capacity(nodes_at_current_level);

        for i in 0..(nodes_at_current_level as u64) {
            let left_key =
                SimpleMerkleNodeKey::new(child_base_key.level, i * 2 + child_base_key.index);

            let v = Hasher::two_to_one(
                &self.get_node_value(&left_key),
                &self.get_node_value(&left_key.sibling()),
            );

            self.set_node_value(left_key.parent(), v);
            child_values.push(v);
        }
        nodes_at_current_level = nodes_at_current_level >> 1;
        child_base_key = child_base_key.parent();

        while child_base_key.level > sub_tree_root_level {
            let mut parent_values = Vec::with_capacity(nodes_at_current_level as usize);
            for i in 0..nodes_at_current_level {
                let parent_key = SimpleMerkleNodeKey::new(
                    child_base_key.level - 1,
                    i as u64 + (child_base_key.index >> 1u64),
                );
                let parent_value =
                    Hasher::two_to_one(&child_values[i * 2], &child_values[i * 2 + 1]);

                self.set_node_value(parent_key, parent_value);
                parent_values.push(parent_value);
            }
            nodes_at_current_level = nodes_at_current_level >> 1;
            child_base_key = child_base_key.parent();
            child_values = parent_values;
        }

        self.rehash_from_node_to_level(child_base_key, 0);

        child_values[0]
    }

    pub fn update_sub_tree(
        &mut self,
        sub_tree_height: u8,
        sub_tree_index: u64,
        sub_tree_offset_index: u64,
        values: &[Hash],
    ) -> anyhow::Result<Hash> {
        let leaves_per_sub_tree = 1u64 << (sub_tree_height as u64);
        if (sub_tree_offset_index + (values.len() as u64)) >= leaves_per_sub_tree {
            anyhow::bail!("cannot set more values in a sub tree than it can contain");
        }
        let offset_index = leaves_per_sub_tree * sub_tree_index + sub_tree_offset_index;
        for (i, v) in values.iter().enumerate() {
            self.set_node_value(
                SimpleMerkleNodeKey::new(self.height, offset_index + i as u64),
                *v,
            );
        }
        Ok(self.rehash_sub_tree(sub_tree_height, sub_tree_index))
    }

    fn _set_sub_tree(
        &mut self,
        sub_tree_height: u8,
        sub_tree_index: u64,
        leaves: &[Hash],
    ) -> anyhow::Result<()> {
        if leaves.len() == 0 {}

        if leaves.len() > (1usize << (sub_tree_height)) {
            anyhow::bail!("cannot set more leaves than can fit in a subtree");
        }

        if sub_tree_height == 0 {
            self.set_leaf(sub_tree_index, leaves[0]);
            return Ok(());
        }

        let offset_index = (1u64 << sub_tree_height) * sub_tree_index;
        for (i, v) in leaves.iter().enumerate() {
            self.set_node_value(
                SimpleMerkleNodeKey::new(self.height, offset_index + i as u64),
                *v,
            );
        }
        self.rehash_sub_tree(sub_tree_height, sub_tree_index);

        Ok(())
    }
    fn _set_sub_tree_dmp(
        &mut self,
        sub_tree_height: u8,
        sub_tree_index: u64,
        leaves: &[Hash],
    ) -> anyhow::Result<DeltaMerkleProofCore<Hash>> {
        if leaves.len() == 0 {
            anyhow::bail!("cannot set a sub tree of 0 length");
        }

        if leaves.len() > (1usize << (sub_tree_height)) {
            anyhow::bail!("cannot set more leaves than can fit in a subtree");
        }

        if sub_tree_height == 0 {
            return Ok(self.set_leaf(sub_tree_index, leaves[0]));
        }

        let offset_index = (1u64 << sub_tree_height) * sub_tree_index;
        for (i, v) in leaves.iter().enumerate() {
            self.set_node_value(
                SimpleMerkleNodeKey::new(self.height, offset_index + i as u64),
                *v,
            );
        }
        
        Ok(self.rehash_sub_tree_dmp(sub_tree_height, sub_tree_index))
    }


    pub fn set_leaf(&mut self, index: u64, value: Hash) -> DeltaMerkleProofCore<Hash> {
        let old_proof = self.get_leaf(index);
        let mut current_value = value;
        let mut current_key = SimpleMerkleNodeKey::new(self.height, index);

        let height = self.height as usize;
        for i in 0..height {
            let new_key = current_key.parent();
            let index = current_key.index;
            self.set_node_value(current_key, current_value);

            current_value = if index & 1 == 0 {
                Hasher::two_to_one(&current_value, &old_proof.siblings[i])
            } else {
                Hasher::two_to_one(&old_proof.siblings[i], &current_value)
            };
            current_key = new_key;
        }
        self.set_node_value(current_key, current_value);
        DeltaMerkleProofCore {
            old_root: old_proof.root,
            old_value: old_proof.value,

            new_root: current_value,
            new_value: value,

            siblings: old_proof.siblings,
            index: index,
        }
    }
    pub fn get_subtree_merkle_proof(
        &self,
        root_level: u8,
        subtree_leaf_node: SimpleMerkleNodeKey,
    ) -> MerkleProofCore<Hash> {
        if root_level > subtree_leaf_node.level {
            panic!("root_level > leaf node level");
        }
        let level_difference = subtree_leaf_node.level - root_level;

        let leaf_key = subtree_leaf_node;
        let value = self.get_node_value(&leaf_key);
        if level_difference == 0 {
            return MerkleProofCore {
                root: value,
                value: value,
                siblings: Vec::new(),
                index: subtree_leaf_node.index,
            };
        }

        let mut current_sibling = leaf_key.sibling();
        let mut siblings = Vec::with_capacity(level_difference as usize);

        while current_sibling.level > root_level {
            siblings.push(self.get_node_value(&current_sibling));
            current_sibling = current_sibling.parent().sibling();
        }

        let root = self.get_node_value(&subtree_leaf_node.parent_at_level(root_level));

        MerkleProofCore {
            index: subtree_leaf_node.index,
            siblings,
            root,
            value,
        }
    }

    pub fn get_leaf_in_subtree(
        &self,
        root_level: u8,
        leaf_level: u8,
        leaf_index: u64,
    ) -> MerkleProofCore<Hash> {
        self.get_subtree_merkle_proof(root_level, SimpleMerkleNodeKey::new(leaf_level, leaf_index))
    }

    pub fn gen_fast_tree_inclusion_proofs(
        height: u8,
        leaves: &[Hash],
    ) -> anyhow::Result<Vec<MerkleProofCore<Hash>>> {
        let max_leaves = (1u64 << (height as u64)) as usize;
        let leaves_count = leaves.len();
        if leaves_count > max_leaves {
            anyhow::bail!("too many leaves for a tree of height {} (tried to add {} leaves, but max is {} leaves for this height)", height, leaves_count, max_leaves);
        } else {
            let mut tmp_tree = Self::new(height);
            for i in 0..leaves_count {
                tmp_tree.set_leaf(i as u64, leaves[i]);
            }

            let inclusion_proofs = (0..leaves_count)
                .map(|i| tmp_tree.get_leaf(i as u64))
                .collect::<Vec<_>>();

            Ok(inclusion_proofs)
        }
    }
}
