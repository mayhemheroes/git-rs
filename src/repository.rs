use std::path::{Path, PathBuf};
use std::collections::HashMap;
use std::vec::Vec;
use glob::glob;

use id::Id;
use reference::Ref;
use error::GitError;
use stores::{Queryable, loose};
use objects::GitObject;
use objects::commit::Commit;

#[derive(Debug)]
pub struct Repository {
    path: PathBuf,
    heads: HashMap<String, Ref>,
    stores: Vec<Box<Queryable>>,
}

impl Repository {
    pub fn from_fs(path: &Path) -> Repository {
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
                    Err(_e) => continue,
                };
                if let Some(item_as_str) = item.to_str() {
                    let name = item_as_str.replace(pb.to_str().unwrap(), "").replace(
                        "/refs/heads/",
                        "",
                    );

                    if let Ok(reference) = Ref::new(item_as_str) {

                        heads.insert(name, reference);
                    }
                }
            }
        }

        let mut repository = Repository {
            path: pb.clone(),
            heads: heads,
            stores: Vec::new(),
        };
        repository.stores.push(Box::new(loose::Store::new()));
        repository
    }

    pub fn path(&self) -> &Path {
        self.path.as_path()
    }

    pub fn rev_parse (&self, what: &str) -> Option<Id> {
        if let Ok(id) = Id::from(what) {
            return Some(id)
        }

        if let Some(head) = self.heads.get(what) {
            return Some(head.to_id())
        }

        return None
    }

    pub fn lookup (&self, what: &str) -> Result<Option<GitObject>, GitError> {
        if let Some(id) = self.rev_parse(what) {
            return self.get_object(&id)
        }
        Ok(None)
    }

    pub fn get_object(&self, id: &Id) -> Result<Option<GitObject>, GitError> {
        for store in &self.stores {
            let result = match store.get(self, id) {
                Ok(v) => v,
                Err(err) => return Err(err),
            };

            if let Some(obj) = result {
                return Ok(Some(obj));
            }
        }
        return Ok(None);
    }

    pub fn get_path_at_commit(&self, what: &Commit, path: Vec<&str>) -> Result<Option<GitObject>, GitError> {
        let tree_id_str = match what.tree() {
            Some(xs) => xs,
            None => return Ok(None)
        };

        let tree_id = Id::from(tree_id_str)?;
        let tree = match self.get_object(&tree_id)? {
            Some(git_object) => {
                match git_object {
                    GitObject::TreeObject(tree) => tree,
                    _ => return Ok(None)
                }
            },
            None => return Ok(None)
        };

        let result = path.iter().fold(Ok(Some(GitObject::TreeObject(tree))), |prev: Result<Option<GitObject>, GitError>, xs| {
            let object = match prev? {
                Some(xs) => xs,
                None => return Ok(None)
            };

            let tree = match object {
                GitObject::TreeObject(tree) => tree,
                _ => return Ok(None)
            };

            let entry = match tree.lookup(xs) {
                Some(xs) => xs,
                None => return Ok(None)
            };

            let next_object = match self.get_object(&entry.id)? {
                Some(git_object) => git_object,
                None => return Ok(None)
            };

            match next_object {
                GitObject::TreeObject(next_tree) => Ok(Some(GitObject::TreeObject(next_tree))),
                GitObject::BlobObject(next_blob) => Ok(Some(GitObject::BlobObject(next_blob))),
                _ => return Ok(None)
            }
        });

        result
    }
}
