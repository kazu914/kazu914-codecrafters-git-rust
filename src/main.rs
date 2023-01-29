use crate::object::Object;
use anyhow::Ok;
use anyhow::Result;

use object::TreeItem;
use object::TreeObject;
use std::env;
use std::fs;
use std::path::Path;
use std::path::PathBuf;

mod object;

enum Command {
    Init,
    CatFile,
    HashObject,
    LsTree,
    WriteTree,
    Unknown,
}

impl Command {
    pub fn from(arg: &str) -> Self {
        match arg {
            "init" => Command::Init,
            "cat-file" => Command::CatFile,
            "hash-object" => Command::HashObject,
            "ls-tree" => Command::LsTree,
            "write-tree" => Command::WriteTree,
            _ => Command::Unknown,
        }
    }
}

fn main() -> Result<()> {
    // Uncomment this block to pass the first stage
    let args: Vec<String> = env::args().collect();
    let command = Command::from(&args[1]);
    let _ = match command {
        Command::Init => execute_init(),
        Command::CatFile => execute_cat_file(&args[3]),
        Command::HashObject => execute_hash_object(&args[3]),
        Command::LsTree => execute_ls_tree(&args[3]),
        Command::WriteTree => execute_write_tree(),
        Command::Unknown => execute_unknown_command(&args[1]),
    };
    Ok(())
}

fn execute_init() -> Result<()> {
    fs::create_dir(".git").unwrap();
    fs::create_dir(".git/objects").unwrap();
    fs::create_dir(".git/refs").unwrap();
    fs::write(".git/HEAD", "ref: refs/heads/master\n").unwrap();
    println!("Initialized git directory");
    Ok(())
}

fn execute_cat_file(hash: &str) -> Result<()> {
    let object = Object::from_hash(hash)?;
    object.print_body()
}

fn execute_hash_object(file_path: &str) -> Result<()> {
    let object = Object::from_target_file(file_path)?;
    object.write_to_file()?;
    let hash = object.get_hash_as_str()?;
    print!("{}", hash);
    Ok(())
}

fn execute_ls_tree(tree_sha: &str) -> Result<()> {
    let object = Object::from_hash(tree_sha)?;
    object.print_body()
}

fn execute_write_tree() -> Result<()> {
    let object = write_tree(Path::new("./"))?;
    let hash_str = object.get_hash_as_str()?;
    print!("{}", hash_str);
    Ok(())
}

fn write_tree(root: &Path) -> Result<Object> {
    let mut tree_object = TreeObject::new();
    let paths = fs::read_dir(root)?;
    let mut entries: Vec<PathBuf> = paths
        .filter(Result::is_ok)
        .map(|e| e.unwrap().path())
        .collect();
    entries.sort();

    for entry in entries {
        let path = entry.as_path();
        if path.starts_with("./.git/") {
            continue;
        }
        if path.is_dir() {
            let hash = write_tree(path)?.get_hash()?;
            let tree_item = TreeItem::new(
                "040000",
                &entry.file_name().unwrap().to_string_lossy(),
                hash,
            );
            tree_object.push(tree_item)?;
        } else {
            let hash = Object::from_target_file(&entry)?.get_hash()?;
            let tree_item = TreeItem::new(
                "100644",
                &entry.file_name().unwrap().to_string_lossy(),
                hash,
            );
            tree_object.push(tree_item)?;
        }
    }

    let object = tree_object.to_object()?;
    object.write_to_file()?;
    Ok(object)
}

fn execute_unknown_command(arg: &str) -> Result<()> {
    println!("unknown command: {}", arg);
    Ok(())
}
