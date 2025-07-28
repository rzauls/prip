use gphoto2::Error;
use gphoto2::file;
use gphoto2::list;
use log::trace;
use std::collections::HashMap;

// TODO: wrap the gphoto2 errors with our own to detach from gphoto2 platform dependencies

pub struct Context {
    inner: gphoto2::Context,
}

impl Context {
    pub fn new() -> Result<Self, Error> {
        let ctx = gphoto2::Context::new()?;
        Ok(Context { inner: ctx })
    }

    pub fn list_cameras(&self) -> Result<Vec<CameraDescriptor>, Error> {
        let mut cameras: Vec<CameraDescriptor> = vec![];
        for cd in self.inner.list_cameras().wait()? {
            trace!("detected {} on port {}", cd.model, cd.port);
            cameras.push(CameraDescriptor::new(cd));
        }

        Ok(cameras)
    }

    pub fn get_camera(self, descriptor: CameraDescriptor) -> Result<Camera, Error> {
        let cam = self.inner.get_camera(&descriptor.descriptor).wait()?;
        Ok(Camera { inner: cam })
    }
}

pub struct CameraDescriptor {
    descriptor: list::CameraDescriptor,
}

impl CameraDescriptor {
    pub fn new(dc: list::CameraDescriptor) -> CameraDescriptor {
        CameraDescriptor { descriptor: dc }
    }
}

impl std::fmt::Display for CameraDescriptor {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{} on port {}",
            self.descriptor.model, self.descriptor.port
        )
    }
}

pub struct Camera {
    inner: gphoto2::Camera,
}

impl Camera {
    pub fn get_port(&self) -> Result<String, Error> {
        Ok(self.inner.port_info()?.path())
    }

    pub fn get_summary(&self) -> Result<String, Error> {
        Ok(self.inner.summary()?)
    }

    pub fn get_folders(&self, root_name: &str) -> Result<FolderContent, Error> {
        let fs = self.inner.fs();
        let folders_iter = fs.list_folders(root_name).wait()?;
        let mut folders = HashMap::with_capacity(folders_iter.len());

        for folder in folders_iter {
            let folder_full_name =
                format!("{}/{folder}", if root_name == "/" { "" } else { root_name });
            folders.insert(folder, self.get_folders(&folder_full_name)?);
        }

        let files = fs.list_files(root_name).wait()?.collect();

        Ok(FolderContent {
            files: files,
            folders: folders,
        })
    }

    pub fn get_file(&self, folder: &str, file: &str) -> Result<File, Error> {
        let fs = self.inner.fs();
        let file = fs.download(folder, file).wait()?;

        Ok(File { inner: file })
    }
}

pub struct File {
    inner: file::CameraFile,
}

impl File {
    pub fn get_filename(&self) -> String {
        self.inner.name()
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
