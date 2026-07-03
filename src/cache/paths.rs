use crate::types::DynResult;
use directories::ProjectDirs;
use std::{
    fs::create_dir_all,
    path::{Path, PathBuf},
};
use tokio::{fs, io::AsyncWriteExt};

pub async fn atomic_write(path: &Path, bytes: &[u8]) -> DynResult<()> {
    let tmp_path = path.with_extension("tmp");
    let mut file = fs::File::create(&tmp_path).await?;

    file.write_all(bytes).await?;
    file.sync_all().await?;
    fs::rename(tmp_path, path).await?;
    Ok(())
}

pub fn get_data_dir() -> PathBuf {
    let proj_dir = ProjectDirs::from("com", "Bit-Barbarian", "MagicVision")
        .expect("Unable to retrieve home directory path!");
    let data_dir = proj_dir.data_dir();
    match data_dir.try_exists() {
        Ok(true) => {}
        Ok(false) => create_dir_all(data_dir).expect("Could not create data directory!"),
        Err(_) => create_dir_all(data_dir).expect("Could not create data directory!"),
    };
    data_dir.to_owned()
}

pub fn get_image_dir() -> PathBuf {
    let image_dir = get_data_dir().join("images/");
    match image_dir.try_exists() {
        Ok(true) => {}
        Ok(false) => create_dir_all(&image_dir).expect("Could not create image directory!"),
        Err(_) => create_dir_all(&image_dir).expect("Could not create image directory!"),
    }
    image_dir
}
