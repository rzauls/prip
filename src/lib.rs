use gphoto2::Error;
use gphoto2::list;
use log::info;
use log::trace;
use std::collections::HashMap;
use std::path::Path;

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

    pub fn move_all_files(
        &self,
        root_name: &str,
        output_dir_root: &Path,
        delete_after_copy: bool,
    ) -> Result<(), Error> {
        let fs = self.inner.fs();
        let folders_iter = fs.list_folders(root_name).wait()?;
        let mut folders = HashMap::with_capacity(folders_iter.len());

        for folder in folders_iter {
            let folder_full_name =
                format!("{}/{folder}", if root_name == "/" { "" } else { root_name });
            folders.insert(
                folder,
                self.move_all_files(&folder_full_name, output_dir_root, delete_after_copy)?,
            );
        }

        // add progress counter without having to use INFO verbosity level
        for file in fs.list_files(root_name).wait()? {
            let output_dir = output_dir_root.join(&file);
            info!(
                "downloading `{}` from `{}` to `{}`",
                &file,
                root_name,
                output_dir.to_str().expect("invalid output path")
            );
            fs.download_to(root_name, &file, &output_dir).wait()?;
            if delete_after_copy {
                fs.delete_file(root_name, &file).wait()?;
                info!("deleted `{}` from `{}`", &file, root_name);
            }
        }

        Ok(())
    }
}
