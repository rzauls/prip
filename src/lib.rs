use gphoto2::Error;
use gphoto2::list;
use log::info;
use log::trace;

use std::path::Path;

// TODO:  wrap the gphoto2 errors with our own to detach from gphoto2 platform dependencies

pub type ProgressCallback = dyn Fn(u64, u64) + Send + Sync;

pub struct FileCount {
    pub total_files: u64,
}

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

    pub fn count_files(&self, root_name: &str) -> Result<FileCount, Error> {
        let total_files = self.count_files_recursive(root_name)?;
        Ok(FileCount { total_files })
    }

    pub fn move_all_files(
        &self,
        root_name: &str,
        output_dir_root: &Path,
        delete_after_copy: bool,
    ) -> Result<(), Error> {
        self.move_all_files_with_callback(root_name, output_dir_root, delete_after_copy, |_, _| {})
    }

    pub fn move_all_files_with_callback<F>(
        &self,
        root_name: &str,
        output_dir_root: &Path,
        delete_after_copy: bool,
        callback: F,
    ) -> Result<(), Error>
    where
        F: Fn(u64, u64) + Send + Sync,
    {
        let file_count = self.count_files(root_name)?;
        let mut processed_files = 0u64;

        self.move_all_files_with_progress(
            root_name,
            output_dir_root,
            delete_after_copy,
            &mut processed_files,
            file_count.total_files,
            &callback,
        )?;

        Ok(())
    }

    fn count_files_recursive(&self, root_name: &str) -> Result<u64, Error> {
        let fs = self.inner.fs();
        let mut total_count = 0u64;

        let files = fs.list_files(root_name).wait()?;
        total_count += files.len() as u64;

        let folders_iter = fs.list_folders(root_name).wait()?;
        for folder in folders_iter {
            let folder_full_name =
                format!("{}/{folder}", if root_name == "/" { "" } else { root_name });
            total_count += self.count_files_recursive(&folder_full_name)?;
        }

        Ok(total_count)
    }

    fn move_all_files_with_progress<F>(
        &self,
        root_name: &str,
        output_dir_root: &Path,
        delete_after_copy: bool,
        processed_files: &mut u64,
        total_files: u64,
        callback: &F,
    ) -> Result<(), Error>
    where
        F: Fn(u64, u64) + Send + Sync,
    {
        let fs = self.inner.fs();
        let folders_iter = fs.list_folders(root_name).wait()?;

        for folder in folders_iter {
            let folder_full_name =
                format!("{}/{folder}", if root_name == "/" { "" } else { root_name });
            self.move_all_files_with_progress(
                &folder_full_name,
                output_dir_root,
                delete_after_copy,
                processed_files,
                total_files,
                callback,
            )?;
        }

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

            *processed_files += 1;
            callback(*processed_files, total_files);
        }

        Ok(())
    }
}
