use core::fmt;
use flate2::write::ZlibEncoder;
use flate2::Compression;
use std::fs;
use std::fs::File;
use std::io::BufReader;
use std::io::Read;
use std::io::Write;
use std::path::Path;

use anyhow::bail;
use anyhow::Ok;
use anyhow::Result;
use flate2::read::ZlibDecoder;
use sha1::{Digest, Sha1};

pub enum ObjectType {
    Blob,
    Tree,
}

impl ObjectType {
    pub fn from(object_type: &str) -> Result<ObjectType> {
        match object_type {
            "blob" => Ok(ObjectType::Blob),
            "tree" => Ok(ObjectType::Tree),
            _ => bail!("Unknown Object Type"),
        }
    }
}

impl fmt::Display for ObjectType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match *self {
            ObjectType::Blob => write!(f, "blob"),
            ObjectType::Tree => write!(f, "tree"),
        }
    }
}

pub struct Object {
    object_type: ObjectType,
    object_size: usize,
    object_body: Vec<u8>,
}

impl Object {
    pub fn from(raw_contents: Vec<u8>) -> Result<Self> {
        let (header, object_body) = Object::detect_header_and_body(raw_contents)?;
        let split_space = header.split_whitespace().collect::<Vec<&str>>();

        let object_type = ObjectType::from(split_space[0])?;
        let object_size: usize = split_space[1].parse().unwrap();

        Ok(Self {
            object_type,
            object_size,
            object_body,
        })
    }

    pub fn from_target_file<P>(file_path: P) -> Result<Self>
    where
        P: AsRef<Path>,
    {
        let target_file_contents = fs::read_to_string(file_path).unwrap();
        Ok(Self {
            object_type: ObjectType::Blob,
            object_size: target_file_contents.len(),
            object_body: target_file_contents.as_bytes().to_vec(),
        })
    }

    pub fn from_hash(hash_str: &str) -> Result<Self> {
        let path = Self::hash_to_object_file_path(hash_str)?;
        Self::from_file(path)
    }

    fn hash_to_object_file_path(hash_str: &str) -> Result<String> {
        let (sub_dir, basename) = hash_str.split_at(2);
        let path = format!(".git/objects/{}/{}", sub_dir, basename);
        Ok(path)
    }

    pub fn print_body(&self) -> Result<()> {
        match self.object_type {
            ObjectType::Blob => {
                print!("{}", String::from_utf8(self.object_body.to_vec()).unwrap());
            }
            ObjectType::Tree => Object::print_tree_object(self.object_body.to_vec())?,
        }
        Ok(())
    }

    fn print_tree_object(body: Vec<u8>) -> Result<()> {
        let tree_object = TreeObject::from(body)?;
        tree_object.print_contents()
    }

    fn detect_header_and_body(raw_contents: Vec<u8>) -> Result<(String, Vec<u8>)> {
        let mut parts = raw_contents.splitn(2, |b| *b == 0);
        let header_bytes = parts.next().unwrap().to_vec();
        let headers_string = String::from_utf8(header_bytes).unwrap();
        let content = parts.next().unwrap().to_vec();
        Ok((headers_string, content))
    }

    pub fn from_file<P>(file_path: P) -> Result<Self>
    where
        P: AsRef<Path>,
    {
        let reader = BufReader::new(File::open(file_path).unwrap());
        let mut deflated_object = Vec::new();
        let mut decoder = ZlibDecoder::new(reader);
        decoder.read_to_end(&mut deflated_object).unwrap();
        Ok(Self::from(deflated_object)?)
    }

    fn get_contents(&self) -> Result<Vec<u8>> {
        let mut contents = Vec::new();
        contents.extend(format!("{} {}\0", self.object_type, self.object_size).as_bytes());
        contents.extend(&self.object_body);
        Ok(contents)
    }

    pub fn get_hash(&self) -> Result<Vec<u8>> {
        let contents = self.get_contents()?;
        let mut hasher = Sha1::default();
        hasher.update(contents);
        let compressed = hasher.finalize();
        Ok(compressed.to_vec())
    }

    pub fn get_hash_as_str(&self) -> Result<String> {
        let hash = self.get_hash()?;
        let mut hash_str = String::new();
        for value in hash {
            hash_str = format!("{}{:02x?}", hash_str, value);
        }
        Ok(hash_str)
    }

    pub fn get_object_file_path(&self) -> Result<String> {
        let hash_str = self.get_hash_as_str()?;
        let (sub_dir, basename) = hash_str.split_at(2);
        let path = format!(".git/objects/{}/{}", sub_dir, basename);
        Ok(path)
    }

    pub fn write_to_file(&self) -> Result<Vec<u8>> {
        let object_file_path = self.get_object_file_path()?;
        let contents = self.get_contents()?;
        let mut encoder = ZlibEncoder::new(Vec::new(), Compression::default());
        let _ = encoder.write(&contents)?;
        let compressed = encoder.finish()?;

        let parent_dir = Path::new(&object_file_path).parent().unwrap();
        fs::create_dir_all(parent_dir)?;
        fs::write(object_file_path, compressed)?;
        Ok(self.get_hash()?)
    }
}

#[derive(Ord, Eq, PartialEq, PartialOrd)]
pub struct TreeItem {
    pub base_name: String,
    _mode: String,
    _hash: Vec<u8>,
}

impl TreeItem {
    pub fn new(mode: &str, base_name: &str, hash: Vec<u8>) -> Self {
        Self {
            base_name: base_name.to_string(),
            _mode: mode.to_string(),
            _hash: hash,
        }
    }

    pub fn as_bytes(&self) -> Result<Vec<u8>> {
        let mut tree_contents: Vec<u8> = Vec::new();

        tree_contents.extend(format!("{} {}\0", self._mode, self.base_name).as_bytes());
        tree_contents.extend(&self._hash);
        Ok(tree_contents)
    }
}

pub struct TreeObject {
    tree_items: Vec<TreeItem>,
}

impl TreeObject {
    pub fn new() -> Self {
        Self {
            tree_items: Vec::new(),
        }
    }

    pub fn from(object_body: Vec<u8>) -> Result<Self> {
        let mut rest_hash_len = 0;
        let mut mode_path = String::new();
        let mut hash = Vec::new();
        let mut tree_object = TreeObject::new();
        for value in object_body {
            if rest_hash_len > 0 {
                hash.push(value);
                rest_hash_len -= 1;
                if rest_hash_len == 0 {
                    let (mode, path) = mode_path.split_once(' ').unwrap();
                    tree_object.push(TreeItem::new(mode, path, hash))?;
                    mode_path = String::new();
                    hash = Vec::new();
                }
                continue;
            }

            if value == 0 {
                rest_hash_len = 20;
                continue;
            }
            mode_path = format!("{}{}", mode_path, value as char);
        }
        Ok(tree_object)
    }

    pub fn push(&mut self, tree_item: TreeItem) -> Result<()> {
        self.tree_items.push(tree_item);
        Ok(())
    }

    pub fn get_contents_as_bytes(&self) -> Result<Vec<u8>> {
        let mut body: Vec<u8> = Vec::new();
        for tree_item in &self.tree_items {
            body.extend(tree_item.as_bytes()?);
        }

        let mut contents = Vec::new();
        contents.extend(format!("tree {}\0", body.len()).as_bytes());
        contents.extend(body);
        Ok(contents)
    }

    pub fn print_contents(&self) -> Result<()> {
        for tree_item in &self.tree_items {
            println!("{}", tree_item.base_name);
        }
        Ok(())
    }

    pub fn to_object(&self) -> Result<Object> {
        Object::from(self.get_contents_as_bytes()?)
    }
}
