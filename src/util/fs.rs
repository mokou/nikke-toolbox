use std::fs;
use std::io::ErrorKind;
use std::os::windows::fs::FileTypeExt;
use std::os::windows::prelude::*;
use std::path::Path;


pub fn fuck_is_symlink<P: AsRef<Path>>(path: P) -> Result<bool, std::io::Error> {
    let metadata = match fs::symlink_metadata(path) {
        Ok(res) => res,
        Err(error) => {
            if error.kind() == ErrorKind::NotFound {
                return Ok(false);
            }
            return Err(error);
        }
    };
    Ok(metadata.file_type().is_symlink_dir())
}

pub fn same_volume(path1: &impl AsRef<Path>, path2: &impl AsRef<Path>) -> Result<bool, std::io::Error> {
    let path_vol = fs::metadata(path1)?.volume_serial_number()
        .ok_or(std::io::ErrorKind::Other)?;
    let local_low_vol = fs::metadata(path2)?.volume_serial_number()
        .ok_or(std::io::ErrorKind::Other)?;
    
    Ok(path_vol == local_low_vol)
}
