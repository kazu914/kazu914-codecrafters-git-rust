use anyhow::Ok;
use anyhow::Result;

pub enum ObjectType {
    Blob,
    Tree,
    Unknown,
}

impl ObjectType {
    pub fn from(object_type: &str) -> Self {
        match object_type {
            "blob" => ObjectType::Blob,
            "tree" => ObjectType::Tree,
            _ => ObjectType::Unknown,
        }
    }
}

pub struct Object {
    object_type: ObjectType,
    _object_size: u32,
    object_body: Vec<u8>,
}

impl Object {
    pub fn from(raw_contents: Vec<u8>) -> Result<Self> {
        let (header, object_body) = Object::detect_header_and_body(raw_contents)?;
        let split_space = header.split_whitespace().collect::<Vec<&str>>();

        let object_type = ObjectType::from(split_space[0]);
        let object_size: u32 = split_space[1].parse().unwrap();

        Ok(Self {
            object_type,
            _object_size: object_size,
            object_body,
        })
    }

    pub fn print_body(&self) -> Result<()> {
        match self.object_type {
            ObjectType::Blob => {
                print!("{}", String::from_utf8(self.object_body.to_vec()).unwrap());
            }
            ObjectType::Tree => Object::print_tree_object(self.object_body.to_vec())?,
            _ => println!("Not implimented"),
        }
        Ok(())
    }

    fn print_tree_object(body: Vec<u8>) -> Result<()> {
        let mut rest_hash_len = 0;
        let mut mode_path = String::new();
        let mut hash = String::new();
        let mut tree_items: Vec<TreeItem> = vec![];
        for value in body {
            if rest_hash_len > 0 {
                hash = format!("{}{:x?}", hash, value);
                rest_hash_len -= 1;
                if rest_hash_len == 0 {
                    let (mode, path) = mode_path.split_once(' ').unwrap();
                    tree_items.push(TreeItem::new(mode, path, &hash));
                    mode_path = String::new();
                    hash = String::new();
                }
                continue;
            }

            if value == 0 {
                rest_hash_len = 20;
                continue;
            }

            mode_path = format!("{}{}", mode_path, value as char);
        }

        for tree_item in tree_items {
            println!("{}", tree_item.base_name);
        }
        Ok(())
    }

    fn detect_header_and_body(raw_contents: Vec<u8>) -> Result<(String, Vec<u8>)> {
        let mut parts = raw_contents.splitn(2, |b| *b == 0);
        let header_bytes = parts.next().unwrap().to_vec();
        let headers_string = String::from_utf8(header_bytes).unwrap();
        let content = parts.next().unwrap().to_vec();
        Ok((headers_string, content))
    }
}

pub struct TreeItem {
    _mode: String,
    pub base_name: String,
    _hash: String,
}

impl TreeItem {
    pub fn new(mode: &str, base_name: &str, hash: &str) -> Self {
        Self {
            _mode: mode.to_string(),
            base_name: base_name.to_string(),
            _hash: hash.to_string(),
        }
    }
}
