use std::fs;
use std::io::Write;
use std::path::Path;
use anyhow::{anyhow, Result};
use serde::{Serialize, Deserialize};
use ignore::WalkBuilder;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileData {
    pub filename: String,
    pub contents: Vec<u8>,
}

impl FileData {
    pub fn from(filename: String, contents: Vec<u8>) -> Self {
        FileData {
            filename: filename,
            contents: contents
        }
    }

    pub fn sizeof(&self) -> usize {
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
    pub files: Vec<FileData>
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

        let walker = WalkBuilder::new(parent_dir_path);

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

    pub fn sizeof(&self) -> usize {
        self.files.iter().map(|fd| fd.sizeof()).sum()
    }

    fn push(&mut self, elem: FileData) {
        self.files.push(elem);
    }

}
