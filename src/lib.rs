use gphoto2::Error;
use gphoto2::list;
use log::trace;
use std::collections::HashMap;

pub struct Context {
    inner: gphoto2::Context,
}

impl Context {
    pub fn new() -> Result<Self, Error> {
        let ctx = gphoto2::Context::new()?;
        Ok(Context { inner: ctx })
    }

    pub fn list_cameras(&self) -> Result<Vec<Camera>, Error> {
        let mut cameras: Vec<Camera> = vec![];
        for cd in self.inner.list_cameras().wait()? {
            trace!("detected {} on port {}", cd.model, cd.port);
            cameras.push(Camera::new(cd));
        }

        Ok(cameras)
    }

    pub fn get_camera(self, descriptor: Camera) -> Result<gphoto2::Camera, Error> {
        self.inner.get_camera(&descriptor.descriptor()).wait()
    }
}

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
    camera: &gphoto2::Camera,
    root_name: &str,
) -> Result<FolderContent, Error> {
    let fs = camera.fs();
    let folders_iter = fs.list_folders(root_name).wait()?;
    let mut folders = HashMap::with_capacity(folders_iter.len());

    for folder in folders_iter {
        let folder_full_name =
            format!("{}/{folder}", if root_name == "/" { "" } else { root_name });
        folders.insert(folder, list_folders_recursive(camera, &folder_full_name)?);
    }

    let files = fs.list_files(root_name).wait()?.collect();

    Ok(FolderContent {
        files: files,
        folders: folders,
    })
}
