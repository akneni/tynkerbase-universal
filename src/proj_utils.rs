
use crate::constants::LINUX_TYNKERBASE_PATH;
use std::fs;
use std::io::Write;
use std::path::{Path, PathBuf};
use anyhow::{anyhow, Result};
use std::env::consts::OS;
use serde::{Serialize, Deserialize};
use ignore::{WalkBuilder, overrides::OverrideBuilder};

#[derive(Debug, Serialize, Deserialize)]
pub struct FileData {
    filename: String,
    contents: Vec<u8>,
}

impl FileData {
    pub fn from(filename: String, contents: Vec<u8>) -> Self {
        FileData {
            filename: filename,
            contents: contents
        }
    }

    pub fn mem_size(&self) -> usize {
        let res = 4*64 + self.filename.len() + self.contents.len();
        return res;
    } 

    pub fn extract(self) -> (String, Vec<u8>) {
        let s = self.filename;
        let d = self.contents;
        (s, d)    
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct FileCollection {
    files: Vec<FileData>
}

impl FileCollection {
    pub fn new() -> Self {
        FileCollection{files: vec![]}
    }
    
    pub fn save(self, output_dir: &str) -> Result<()> {
        // Given a dictionary of paths and their content in bytes as well as the path for the parent 
        // directory, this function will create all the files on the disk.
        // If an error occurs, the function will stop copying the files to the disk and return the error
        for file_obj in self.files {
            let file_name= &file_obj.filename;
            let data = &file_obj.contents;
            // (file_name, data) 
            let full_path = std::path::Path::new(output_dir).join(file_name);
    
            if let Some(parent) = full_path.parent() {
                if !parent.exists() {
                    let res = std::fs::create_dir_all(&parent)
                        .map_err(|e| e.to_string());
                    if res.is_err() {
                        return Err(anyhow!("{:?}", res));
                    }
                }
            }
    
            let mut outfile = match fs::File::create(&full_path) {
                Ok(f) => f,
                Err(e) => return Err(anyhow!("{}", e)),
            };
            let res = outfile.write_all(&data);
            if res.is_err() {
                return res.map_err(|e| anyhow!("{}", e));
            }
        }

        Ok(())
    }    

    pub fn load(parent_dir: &str, ignore: &Vec<String>) -> Result<Self> {
        let mut res = FileCollection { files: vec![] };

        let parent_dir_path = Path::new(parent_dir);
        if parent_dir_path.is_file() {
            return Err(anyhow!("Argument must be a directory, not a file."));
        }

        let mut walker = WalkBuilder::new(parent_dir_path);
        walker.git_ignore(false);

        let mut override_obj = OverrideBuilder::new(parent_dir);

        for s in ignore.iter() {
            override_obj.add(&s).unwrap();
        }

        walker.overrides(override_obj.build().unwrap());

        for path in walker.build() {
            if let Ok(entry) = path {
                if entry.file_type().map_or(false, |ft| ft.is_file()) {
                    let p = entry.path();
                    let bin = fs::read(p).unwrap();
                    let p_str = p.to_str().unwrap().to_string();
                    res.push(FileData::from(p_str, bin));
                }
            }
        }

        Ok(res)
    }

    fn push(&mut self, elem: FileData) {
        self.files.push(elem);
    }
}


pub fn create_proj(name: &str) -> Result<String> {
    if OS == "linux" {
        let path_str = format!("{LINUX_TYNKERBASE_PATH}/{name}");
        let path = Path::new(&path_str);
        if !Path::exists(&path) {
            return Err(anyhow!("Project `{}` already exists", name));
        }
        if let Err(e) = fs::create_dir(&path) {
            return Err(anyhow!("Error creating dir: `{}`", e));
        }
        return Ok(format!("Created `{LINUX_TYNKERBASE_PATH}/{name}`"));
    }
    Err(anyhow!("OS `{}` is unsupported", OS))
}

pub fn add_files_to_proj(name: &str, files: FileCollection) -> Result<()> {
    if !get_proj_names().contains(&name.to_string()) {
        return Err(anyhow!("Project `{}` does not exist.", {name}));
    }

    if OS == "linux" {
        let proj_path = format!("{LINUX_TYNKERBASE_PATH}/{name}");
        if let Err(e) = files.save(&proj_path) {
            return Err(anyhow!("{}", e));
        }
    }
    Err(anyhow!("OS `{}` is unsupported", OS))
}

pub fn get_proj_names() -> Vec<String> {
    // traverses the tynkerbase-projects directory to get all the names of all the folders
    // (which should each contain a project)
    let projects = fs::read_dir(format!("{LINUX_TYNKERBASE_PATH}"));
    match projects {
        Ok(projects) => {
            let mut res = vec![];
            for path in projects {
                if let Ok(path) = path {
                    if let Ok(path) = path.file_name().into_string() {
                        res.push(path);
                    }
                }
            }
            res
        }
        _ => vec![]
    }
}

pub fn delete_proj(name: &str) -> Result<()> {
    if OS == "linux" {
        let path = format!("{LINUX_TYNKERBASE_PATH}/{name}");
        if !Path::new(&path).exists() {
            return Err(anyhow!("Project does not exist"));
        }
        if let Err(e) = fs::remove_dir_all(path) {
            return Err(anyhow!("{}", e));
        }
    }
    Ok(())
}

pub fn clear_proj(name: &str) -> Result<()> {
    let res = delete_proj(name);
    if res.is_err() {
        return res;
    }
    let res = create_proj(name);
    if let Err(e) = res {
        return Err(anyhow!("{}", e));
    }
    Ok(())
}

pub fn load_proj_files(name: &str, ignore: Option<&Vec<String>>) -> Result<FileCollection> {
    let path_str = format!("{}/{}", LINUX_TYNKERBASE_PATH, name);
    let empty_vec: Vec<String> = vec![];
    let ignore = ignore.unwrap_or(&empty_vec);

    match FileCollection::load(&path_str, &ignore) {
        Ok(fc) => Ok(fc),
        Err(e) => Err(e),
    }
}