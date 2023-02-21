#![feature(windows_by_handle)]
#![feature(io_error_more)]
use dialoguer::{console::Term, theme::ColorfulTheme, Select};
use fs_extra::dir::{CopyOptions, move_dir};
use rfd::FileDialog;
use std::fs;
use std::io::ErrorKind;
use std::os::windows::prelude::*;
use std::os::windows::fs::{FileTypeExt, symlink_dir};
use winreg::enums::*;
use winreg::RegKey;
use lazy_static::lazy_static;
use std::path::{PathBuf, Path};

const HKCU: RegKey = RegKey::predef(HKEY_CURRENT_USER);

const RELOC_SUFFIX: &str = r"nikke-toolbox\LocalLow";
const CPB_SUFFIX: &str = "com_proximabeta";
const CPBN_SUFFIX: &str = r"Unity\com_proximabeta_NIKKE";

lazy_static! {
    static ref INSTALL_PATH: PathBuf = PathBuf::from(
        HKCU.open_subkey(r"Software\Microsoft\Windows\CurrentVersion\Uninstall\nikke_launcher")
        .unwrap_or(HKCU)
        .get_value::<String, &str>("GameInstallPath")
        .unwrap_or_default());

    static ref LOCAL_LOW: PathBuf = dirs::home_dir().unwrap().join("AppData").join("LocalLow");

    static ref CPB: PathBuf =  LOCAL_LOW.join(CPB_SUFFIX);
    static ref CPBN: PathBuf = LOCAL_LOW.join(CPBN_SUFFIX);

    static ref COPY_OPTIONS: CopyOptions = CopyOptions::new().copy_inside(true);

}

fn main() -> std::io::Result<()> {
    let items = vec!["Exit", "Relocate Nikke", "Undo Nikke Relocation", "Nuke Installation", "About"];
    loop {
        let selection = Select::with_theme(&ColorfulTheme::default())
            .with_prompt("Nikke Toolbox v0.1")
            .items(&items)
            .default(0)
            .interact_on_opt(&Term::stderr())?;

        match selection {
            Some(index) => match index {
                1 => relocate()?,
                2 => undo_relocate()?,
                3 => nuke()?,
                4 => about(),
                _ => break
            }
            None => break
        }
    }

    Ok(())
}

fn relocate() -> std::io::Result<()> {
    let cpb =  CPB.as_path();
    let cpbn = CPBN.as_path();
    
    // TODO: Make is_relocated() to check relocation status more throughly
    if fuck_is_symlink(cpb)? || fuck_is_symlink(cpbn)? {
        println!("Already relocated.");
        return Ok(());
    }

    let items = vec!["Cancel", "Confirm", "Select different location"];
    let mut install = PathBuf::from(INSTALL_PATH.as_path());

    loop {
        let selection = Select::with_theme(&ColorfulTheme::default())
        .with_prompt(format!("Relocating to {}", install.display()))
        .items(&items)
        .default(0)
        .interact_on_opt(&Term::stderr()).unwrap();
    
        match selection {
            Some(index) => match index {
                1 => break,
                2 => {
                    install = match FileDialog::new()
                        .set_title("Select Nikke install location")
                        .set_directory(install.as_path())
                        .pick_folder() {
                            Some(path) => path,
                            None => install
                        };
                    }
                _ => return Ok(())
            }
            None => return Ok(())
        }
    }
    
    if same_volume(&install)? {
        println!("Game install path is on same filesystem as: {}.\nNo need to relocate.", LOCAL_LOW.display());
        return Ok(())
    }

    let reloc_path = install.join(RELOC_SUFFIX);
    let cpb_reloc = reloc_path.join(CPB_SUFFIX);
    let cpbn_reloc = reloc_path.join(CPBN_SUFFIX);

    if let Err(error) = fs::create_dir_all(reloc_path.join("Unity")) {
        println!("Problem creating destination directory: {:#?}", error);
        return Ok(());
    }

    println!("Relocating Nikke. This may take a while...");
    
    println!("Moving files from: {}\nto: {}", &cpb.display(), &cpb_reloc.display());
    if !cpb.exists() { fs::create_dir(cpb)?; } 
    move_dir(cpb, &cpb_reloc, &COPY_OPTIONS).unwrap_or_else(|error| {
        println!("Problem copying files to destination directory: {:#?}", error);
        0
    });
    symlink_dir(&cpb_reloc, cpb).unwrap_or_else(|error| {
        println!("Problem creating symlink: {:#?}", error);
    });

    println!("Moving files from {}\nto: {}", &cpbn.display(), &cpbn_reloc.display());
    if !cpbn.exists() { fs::create_dir(cpbn)?; }
    move_dir(cpbn, &cpbn_reloc, &COPY_OPTIONS).unwrap_or_else(|error| {
        println!("Problem copying files to destination directory: {:#?}", error);
        0
    });
    symlink_dir(&cpbn_reloc, cpbn).unwrap_or_else(|error| {
        println!("Problem creating symlink: {:#?}", error);
    });
    
    println!("Nikke has been relocated.");
    Ok(())
}

fn undo_relocate() -> std::io::Result<()> {
    let cpb = CPB.as_path();
    let cpbn = CPBN.as_path();
    let cpb_reloc = fs::read_link(cpb).unwrap_or(PathBuf::from(cpb));
    let cpbn_reloc = fs::read_link(cpbn).unwrap_or(PathBuf::from(cpbn));

    if !fuck_is_symlink(cpb)? {
        println!("No relocation to undo at: {}", &cpb.display());
    } else {
        fs::remove_dir(cpb).unwrap_or_else(|error| {
            println!("Unable to remove symlink: {:#?}", error);
        });
        println!("Moving files from: {}\nto: {}", &cpb_reloc.display(), &cpb.display());
        move_dir(&cpb_reloc, cpb, &COPY_OPTIONS).unwrap_or_else(|error| {
            println!("Unable to move files: {:#?}", error);
            1
        });
        match fs::remove_dir(cpb) {
            Ok(_) => (),
            Err(error) => {
                if error.kind() != ErrorKind::DirectoryNotEmpty { return Err(error); }
            }
        }
    }
    
    if !fuck_is_symlink(cpbn)? {
        println!("No relocation to undo at: {}", &cpbn.display());
    } else {
        fs::remove_dir(cpbn).unwrap_or_else(|error| {
            println!("Unable to remove symlink: {:#?}", error);
        });
        println!("Moving files from: {}\nto: {}", &cpbn_reloc.display(), &cpbn.display());
        move_dir(&cpbn_reloc, cpbn, &COPY_OPTIONS).unwrap_or_else(|error| {
            println!("Unable to move files: {:#?}", error);
            1
        });
        match fs::remove_dir(cpbn) {
            Ok(_) => (),
            Err(error) => {
                if error.kind() != ErrorKind::DirectoryNotEmpty { return Err(error); }
            }
        }
    }

    fs::remove_dir(cpbn_reloc.parent().unwrap()).unwrap_or_else(|error| {
        if error.kind() != ErrorKind::DirectoryNotEmpty {
            println!("Problem cleaning up: {:#?}", error);
        };
    });
    fs::remove_dir(cpb_reloc.parent().unwrap()).unwrap_or_else(|error| {
        if error.kind() != ErrorKind::DirectoryNotEmpty {
            println!("Problem cleaning up: {:#?}", error);
        };
    });

    println!("Relocation undone.");
    Ok(())
}

fn nuke() -> std::io::Result<()> {
    let items = vec!["Yes", "No"];
    let selection = Select::with_theme(&ColorfulTheme::default())
        .with_prompt("Are you sure you want to delete several gigabytes of game data?")
        .items(&items)
        .default(1)
        .interact_on_opt(&Term::stderr()).unwrap();

    match selection {
        Some(index) => match index {
            0 => (),
            _ => return Ok(())
        }
        None => return Ok(())
    }

    println!("Deleting...");
    let mut cpb = PathBuf::from(CPB.as_path());
    let mut cpbn = PathBuf::from(CPBN.as_path());
    
    if fuck_is_symlink(&cpb)? {
        cpb = fs::read_link(&cpb)?;
    }
    match fs::remove_dir_all(&cpb) {
        Ok(res) => {
            println!("Nuked: {}", &cpb.display());
            res
        },
        Err(error) => {
            if error.kind() == ErrorKind::NotFound {
                println!("Nothing to nuke at: {}", &cpb.display());
            } else {
                return Err(error);
            }
        }
    }
    if fuck_is_symlink(CPB.as_path())? {
        fs::create_dir(&cpb)?;
    }

    if fuck_is_symlink(&cpbn)? {
        cpbn = fs::read_link(&cpbn)?;

    }
    match fs::remove_dir_all(&cpbn) {
        Ok(res) => {
            println!("Nuked: {}", &cpbn.display());
            res
        },
        Err(error) => {
            if error.kind() == ErrorKind::NotFound {
                println!("Nothing to nuke at: {}", &cpbn.display());
            } else {
                return Err(error);
            }
        }
    }
    if fuck_is_symlink(CPBN.as_path())? {
        fs::create_dir(&cpbn)?;
    }
    Ok(())
}

fn about() {
    println!{r#"
A helpful set of tools to manage your Nikke installation.

-------------------------------------------------------------------------------

HelmSTAR Icon:
Copyright (c) 2022 Irene#7777. All rights reserved.

-------------------------------------------------------------------------------

Everything Else:
Copyright (c) 2023 Mokou <nikke-toolbox@mokou.io>

Permission is hereby granted, free of charge, to any person obtaining a copy
of this software and associated documentation files (the "Software"), to deal
in the Software without restriction, including without limitation the rights
to use, copy, modify, merge, publish, distribute, sublicense, and/or sell
copies of the Software, and to permit persons to whom the Software is
furnished to do so, subject to the following conditions:

The above copyright notice and this permission notice shall be included in all
copies or substantial portions of the Software.

THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR
IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY,
FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE
AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER
LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM,
OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN THE
SOFTWARE.

-------------------------------------------------------------------------------

Source code available at: https://github.com/mokou/nikke-toolbox
"#}
}

fn fuck_is_symlink<P: AsRef<Path>>(path: P) -> Result<bool, std::io::Error> {
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

fn same_volume(path: &PathBuf) -> Result<bool, std::io::Error> {
    let path_vol = fs::metadata(path)?.volume_serial_number()
        .ok_or(std::io::ErrorKind::Other)?;
    let local_low_vol = fs::metadata(LOCAL_LOW.as_path())?.volume_serial_number()
        .ok_or(std::io::ErrorKind::Other)?;
    
    Ok(path_vol == local_low_vol)
}
