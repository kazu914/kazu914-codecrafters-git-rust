use crate::object::Object;
use anyhow::Ok;
use anyhow::Result;
use flate2::read::ZlibDecoder;
use flate2::write::ZlibEncoder;
use flate2::Compression;
use sha1::{Digest, Sha1};
use std::env;
use std::fs::File;
use std::io::BufReader;
use std::io::Write;
use std::path::Path;
use std::{fs, io::Read};

mod object;

enum Command {
    Init,
    CatFile,
    HashObject,
    LsTree,
    Unknown,
}

impl Command {
    pub fn from(arg: &str) -> Self {
        match arg {
            "init" => Command::Init,
            "cat-file" => Command::CatFile,
            "hash-object" => Command::HashObject,
            "ls-tree" => Command::LsTree,
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
    let file_path = get_object_file_path(hash)?;
    let contents = read_object_file_contents(&file_path)?;
    let object = Object::from(contents)?;
    object.print_body()
}

fn execute_hash_object(file_path: &str) -> Result<()> {
    let contents = fs::read_to_string(file_path).unwrap();
    let blob_contents = format!("blob {}\0{}", contents.len(), contents);
    let blob_hash = calculate_blob_hash(&blob_contents)?;
    write_object_file(&blob_hash, &blob_contents)?;
    println!("{}", blob_hash);
    Ok(())
}

fn write_object_file(blob_hash: &str, blob_contents: &str) -> Result<()> {
    let object_file_path = get_object_file_path(blob_hash)?;

    let mut encoder = ZlibEncoder::new(Vec::new(), Compression::default());
    let _ = encoder.write(blob_contents.as_bytes())?;
    let compressed = encoder.finish()?;

    let parent_dir = Path::new(&object_file_path).parent().unwrap();
    fs::create_dir_all(parent_dir)?;
    fs::write(object_file_path, compressed)?;
    Ok(())
}

fn calculate_blob_hash(blob_contents: &str) -> Result<String> {
    let mut hasher = Sha1::default();
    hasher.update(blob_contents);
    let compressed = hasher.finalize();
    let s = format!("{:02x}", compressed);
    Ok(s)
}

fn execute_ls_tree(tree_sha: &str) -> Result<()> {
    let object_file_path = get_object_file_path(tree_sha)?;
    let contents = read_object_file_contents(&object_file_path)?;

    let object = Object::from(contents)?;
    object.print_body()
}

fn execute_unknown_command(arg: &str) -> Result<()> {
    println!("unknown command: {}", arg);
    Ok(())
}

fn get_object_file_path(object: &str) -> Result<String> {
    let (sub_dir, basename) = object.split_at(2);
    let path = format!(".git/objects/{}/{}", sub_dir, basename);
    Ok(path)
}

fn read_object_file_contents(file_path: &str) -> Result<Vec<u8>> {
    let reader = BufReader::new(File::open(file_path).unwrap());
    let mut deflated_object = Vec::new();
    let mut decoder = ZlibDecoder::new(reader);
    decoder.read_to_end(&mut deflated_object).unwrap();
    Ok(deflated_object)
}
