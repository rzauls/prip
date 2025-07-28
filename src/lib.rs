use anyhow::Result;
use gphoto2::filesys;
use std::collections::HashMap;

pub fn find_matches(content: &str, pattern: &str, mut writer: impl std::io::Write) -> Result<()> {
    if content.contains(pattern) {
        writeln!(writer, "{}", content)?;
    };

    Ok(())
}

#[test]
fn find_a_match() {
    let mut result = Vec::new();
    let _ = find_matches("lorem ipsum", "lorem", &mut result);
    assert_eq!(result, b"lorem ipsum\n");
}

#[test]
fn dont_find_a_match() {
    let mut result: Vec<u8> = Vec::new();
    let _ = find_matches("lorem ipsum", "loreal", &mut result);
    assert_eq!(result, b"");
}

#[derive(Debug)]
#[allow(dead_code)]
pub struct FolderContent {
    folders: HashMap<String, FolderContent>,
    files: Vec<String>,
}

pub fn list_directory_recursive(
    fs: &filesys::CameraFS,
    dir_name: &str,
) -> gphoto2::Result<FolderContent> {
    let folders_iter = fs.list_folders(dir_name).wait()?;
    let mut folders = HashMap::with_capacity(folders_iter.len());

    for folder in folders_iter {
        let folder_full_name = format!("{}/{folder}", if dir_name == "/" { "" } else { dir_name });
        folders.insert(folder, list_directory_recursive(fs, &folder_full_name)?);
    }

    let files = fs.list_files(dir_name).wait()?.collect();

    Ok(FolderContent {
        files: files,
        folders: folders,
    })
}
