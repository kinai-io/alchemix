use std::{
    fs::{self, File},
    io::{Read, Write},
    path::{Path, PathBuf},
};

use serde::{de::DeserializeOwned, Serialize};

pub fn load<P: AsRef<Path>, T: DeserializeOwned>(file: P) -> Result<T, ()> {
    let mut file = File::open(&file).expect(&format!("unable to read file {:?}", file.as_ref()));
    let mut string_content = String::new();
    file.read_to_string(&mut string_content)
        .expect(&format!("unable to read file content{:?}", file));
    let json: T = serde_json::from_str(&string_content).expect("Malformed json");
    Ok(json)
}

pub fn write<P: AsRef<Path>, T: Serialize>(file: P, data: &T) -> Result<(), ()> {
    let res = serde_json::to_string_pretty(data);
    if let Ok(json_string) = res {
        let pathbuf = PathBuf::from(file.as_ref());
        if let Some(parent) = pathbuf.parent() {
            fs::create_dir_all(parent).expect("Unable to create db directory");
        }
        let mut json_file: File = File::create(file).expect("No Error");
        if let Ok(_) = write!(&mut json_file, "{}", json_string) {
            Ok(())
        } else {
            Err(())
        }
    } else {
        Err(())
    }
}