use crate::types::DynResult;
use std::path::Path;
use tokio::{fs, io::AsyncWriteExt};

pub async fn atomic_write(path: &Path, bytes: &[u8]) -> DynResult<()> {
    let tmp_path = path.with_extension("tmp");
    let mut file = fs::File::create(&tmp_path).await?;

    file.write_all(bytes).await?;
    file.sync_all().await?;
    fs::rename(tmp_path, path).await?;
    Ok(())
}
