extern crate byteorder;

use std::mem;

use std::str;

use std::io::prelude::*;
use std::io::{self, SeekFrom, BufWriter, BufReader};
use std::fs::{File, OpenOptions};

use self::byteorder::{LittleEndian, WriteBytesExt, ReadBytesExt};

static PAGE_SIZE: u32 = 4096;



struct TreeNode {
    index: u32,
    max_members: u32,
    members: Vec<TreeKey>,
    children: Vec<u32>,
}

impl TreeNode {
    fn new(ix: u32, max_members: u32) -> TreeNode {
        TreeNode {
            index: ix,
            max_members: max_members,
            members: Vec::with_capacity(max_members as usize),
            children: Vec::with_capacity((max_members+1) as usize),
        }
    }

    fn from_disk(f: &mut File, max_members: u32, index: u32) -> Result<TreeNode, io::Error> {
        let offset = size_on_disk(max_members) * index;

        f.seek(SeekFrom::Start(offset as u64))?;
        let mut buf = BufReader::new(f);

        let num_children = buf.read_u32::<LittleEndian>()?;
        let num_members = buf.read_u32::<LittleEndian>()?;

        let mut children_indexes = Vec::with_capacity(num_children as usize);
        for _ in 0..num_children {
            children_indexes.push(buf.read_u32::<LittleEndian>()?);
        }

        let mut members = Vec::with_capacity(num_members as usize);
        for _ in 0..num_members {
            let mut char_buffer = [0u8; 4096];
            buf.read_exact(&mut char_buffer)?;
            let key = str::from_utf8(&char_buffer).unwrap().to_string();

            buf.read_exact(&mut char_buffer)?;
            let value = str::from_utf8(&char_buffer).unwrap().to_string();
            members.push(TreeKey { key: key, value: value });
        }

        Ok(TreeNode {
            index: index,
            max_members: max_members,
            members: members,
            children: children_indexes,
        })
    }

    fn to_disk(&self, mut f: &File) -> Result<(), io::Error> {
        /*
        Structure on disk:
        u32                            : number of children
        u32                            : number of key-value pairs
        u32 * (self.size)              : child indexes
        u8 - PAGE_SIZE * self.size * 2 : keys and values for this node
        */

        let seek_loc = self.index * size_on_disk(self.max_members);
        f.seek(SeekFrom::Start(seek_loc as u64))?;
        let mut buf = BufWriter::new(f);

        buf.write_u32::<LittleEndian>(self.children.len() as u32)?;
        buf.write_u32::<LittleEndian>(self.members.len() as u32)?;

        for child in self.children.iter() {
            buf.write_u32::<LittleEndian>(*child)?;
        }

        let right_pad_children = self.max_members - (self.children.len() as u32) + 1;
        for _ in 0..right_pad_children {
            buf.write_u32::<LittleEndian>(0)?;
        }

        for member in self.members.iter() {
            self.write_member_to_disk(&mut buf, &member)?;
        }

        let right_pad_members = self.max_members - (self.members.len() as u32);
        for i in 0..right_pad_members {
            let padding = [0u8;8192];

            buf.write(&padding)?;
        }

        Ok(())
    }

    fn write_member_to_disk(&self, b: &mut BufWriter<&File>, m: &TreeKey) -> Result<(), io::Error> {
        let mut key_write_buffer = [0u8;4096];
        let mut value_write_buffer = [0u8;4096];

        let key_bytes = m.key.as_bytes();
        let value_bytes = m.value.as_bytes();

        for (to_write, key_byte) in key_write_buffer.iter_mut().zip(key_bytes.iter()) {
            *to_write = *key_byte;
        }

        for (to_write, val_byte) in value_write_buffer.iter_mut().zip(value_bytes.iter()) {
            *to_write = *val_byte;
        }

        b.write(&key_write_buffer)?;
        b.write(&value_write_buffer)?;

        Ok(())
    }

    fn insert(&mut self, key: &str, value: &str) {
        if self.members.len() == self.max_members as usize {
            panic!("Insert called on full node.");
        }

        let mut insert_index = 0;

        for (i, member) in self.members.iter().enumerate() {
            if key > &member.key[..] {
                insert_index = i;
                break;
            }
        }
        self.members.insert(insert_index, TreeKey { key: key.to_string(), value: value.to_string() });
    }
}

struct TreeKey {
    key: String,
    value: String,
}

pub struct BTree {
    tree_file: File,
    root_node: TreeNode,
    max_members: u32,
}

impl BTree {
    pub fn open(filename: &str) -> Result<BTree, io::Error> {
        let f = File::open(filename)?;

        Ok(
            BTree {
                tree_file: f,
                root_node: TreeNode::new(0, 2),
                max_members: 2
            }
        )
    }

    pub fn new(filename: &str) -> Result<BTree, io::Error> {
        let f = OpenOptions::new().write(true)
            .create_new(true)
            .read(true)
            .open(filename)?;

        Ok(
            BTree {
                tree_file: f,
                root_node: TreeNode::new(0,5),
                max_members: 5
            }
        )
    }

    pub fn insert(&mut self, key: &str, value: &str) -> Result<(), io::Error> {
        let mut insert_node = match self.search_for_insert(key, 0) {
            Ok(node) => node,
            Err(_) => TreeNode::new(0, self.max_members),
        };

        insert_node.insert(key, value);
        insert_node.to_disk(&mut self.tree_file)?;

        Ok(())
    }

    fn search_for_insert(&mut self, key: &str, index: u32) -> Result<TreeNode, io::Error> {
        let current_node = TreeNode::from_disk(&mut self.tree_file, self.max_members, index)?;

        if current_node.children.len() == 0 {
            if current_node.members.len() < self.max_members as usize {
                return Ok(current_node);
            } else {
                panic!();
            }
        }

        panic!();
    }
}

fn size_on_disk(max_members: u32) -> u32 {
    // Two size indicators. How many children and how many values
    let size_of_sizes = mem::size_of::<u32>() as u32 * 2;

    // Size of maximum number of children
    let size_of_child_indexes = mem::size_of::<u32>() as u32 * max_members + 1;

    // Size of maximum number of members
    let size_of_members = (PAGE_SIZE * 2) * max_members;

    size_of_sizes + size_of_child_indexes + size_of_members
}


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn serialize_to_and_from_disk () {
        let mut tree = BTree::new("test.db").unwrap();
        tree.root_node.members = vec!(TreeKey{key: "key1".to_string(), value: "value1".to_string()});
        tree.root_node.children = vec!(1, 2, 3);

        tree.root_node.to_disk(&tree.tree_file).unwrap();

        let node = TreeNode::from_disk(&mut tree.tree_file, 5, 0).unwrap();

        assert_eq!(node.members.len(), 1);
        assert_eq!(node.children.len(), 3);
    }
}
