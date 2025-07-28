use gphoto2::filesys;
use gphoto2::list;
use std::collections::HashMap;

pub struct Camera {
    descriptor: list::CameraDescriptor,
}

impl Camera {
    pub fn new(dc: list::CameraDescriptor) -> Camera {
        Camera { descriptor: dc }
    }
    pub fn descriptor(self) -> list::CameraDescriptor {
        self.descriptor
    }
}

impl std::fmt::Display for Camera {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{} on port {}",
            self.descriptor.model, self.descriptor.port
        )
    }
}

pub struct FolderContent {
    folders: HashMap<String, FolderContent>,
    files: Vec<String>,
}

impl std::fmt::Display for FolderContent {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        for (name, fc) in &self.folders {
            write!(f, "{}/\n", name).expect("invalid folder");
            let _ = fc.fmt(f);
        }
        if self.files.len() > 0 {
            for file in &self.files {
                write!(f, "-{}\n", file).expect("invalid filename");
            }
        }

        Ok(())
    }
}

pub fn list_folders_recursive(
    fs: &filesys::CameraFS,
    dir_name: &str,
) -> gphoto2::Result<FolderContent> {
    let folders_iter = fs.list_folders(dir_name).wait()?;
    let mut folders = HashMap::with_capacity(folders_iter.len());

    for folder in folders_iter {
        let folder_full_name = format!("{}/{folder}", if dir_name == "/" { "" } else { dir_name });
        folders.insert(folder, list_folders_recursive(fs, &folder_full_name)?);
    }

    let files = fs.list_files(dir_name).wait()?.collect();

    Ok(FolderContent {
        files: files,
        folders: folders,
    })
}
