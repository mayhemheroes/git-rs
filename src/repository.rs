use std::path::{Path, PathBuf};
use std::collections::HashMap;
use reference::Ref;
use glob::glob;

#[derive(Debug)]
pub struct Repository {
    path: PathBuf,
    heads: HashMap<String, Ref>
}

impl Repository {
    pub fn new (path: &Path) -> Repository {
        let mut heads = HashMap::new();
        let pb = PathBuf::from(path);

        let mut glob_path = pb.clone();
        glob_path.push("refs");
        glob_path.push("heads");
        glob_path.push("*");

        if let Some(glob_path_str) = glob_path.to_str() {
            for entry in glob(glob_path_str).expect("Weena wonga") {
                let item = match entry {
                    Ok(item) => item,
                    Err(_e) => continue
                };
                if let Some(item_as_str) = item.to_str() {
                    let name = item_as_str.replace(pb.to_str().unwrap(), "").replace("/refs/heads/", "");

                    if let Ok(reference) = Ref::new(item_as_str) {

                        heads.insert(name, reference);
                    }
                }
            }
        }

        Repository {
            path: pb.clone(),
            heads: heads
        }
    }
}
