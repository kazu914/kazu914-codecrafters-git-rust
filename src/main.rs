use anyhow::Ok;
use anyhow::Result;
use bytes::buf::BufExt;
use flate2::read::ZlibDecoder;
#[allow(unused_imports)]
use std::env;
#[allow(unused_imports)]
use std::fs;
use std::io::Read;

const NULL_CHAR: char = '\0';

enum Command {
    Init,
    CatFile,
    Unknown,
}

impl Command {
    pub fn from(arg: &str) -> Self {
        match arg {
            "init" => Command::Init,
            "cat-file" => Command::CatFile,
            _ => Command::Unknown,
        }
    }
}

enum ObjectType {
    Blob,
    Unknown,
}

impl ObjectType {
    pub fn from(object_type: &str) -> Self {
        match object_type {
            "blob" => ObjectType::Blob,
            _ => ObjectType::Unknown,
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
    handle_object(&contents)
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

fn read_object_file_contents(file_path: &str) -> Result<String> {
    let contents = fs::read(file_path).unwrap();
    let mut deflated_object = String::new();
    let _ = ZlibDecoder::new(contents.reader())
        .read_to_string(&mut deflated_object)
        .unwrap();
    Ok(deflated_object)
}

fn handle_object(contents: &str) -> Result<()> {
    let split_null = split_at_null_char(contents);
    let split_space = split_null[0].split_whitespace().collect::<Vec<&str>>();
    let object_type = ObjectType::from(split_space[0]);
    match object_type {
        ObjectType::Blob => {
            print!("{}", split_null[1]);
        }
        _ => println!("Not implimented"),
    }
    Ok(())
}

fn split_at_null_char(target: &str) -> Vec<&str> {
    target.split(NULL_CHAR).collect::<Vec<&str>>()
}
