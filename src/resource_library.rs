use std::{collections::BTreeMap, fs::File, io::{Read, Write}};

use anyhow::{anyhow, bail, Result};
use serde::{Deserialize, Serialize};

const FORBIDDEN_CHARACTERS: &'static str = "\\?%*:|\"<>,;=";
const HEADER_BYTES: [u8; 10] = [0x67, 0xD7, 0x70, 0x3A, 0x54, 0x3D, 0xDB, 0xF5, 0x17, 0x95]; // This is just a string of random numbers, it has no real signifigance

fn verify_str(str: &str) -> Result<&str> {
    for c in str.chars() {
        for forbidden in FORBIDDEN_CHARACTERS.chars() {
            if c == forbidden {
                bail!("Character '{}' not allowed in path.", c);
            }
        }
    }

    Ok(str)
}

fn verify_string(string: String) -> Result<String> {
    for c in string.chars() {
        for forbidden in FORBIDDEN_CHARACTERS.chars() {
            if c == forbidden {
                bail!("Character '{}' not allowed in path.", c);
            }
        }
    }

    Ok(string)
}
#[derive(Debug, PartialEq, Eq, Clone)]
pub struct ResourceLibrary {
    map: BTreeMap<String, Box<[u8]>>
}

impl ResourceLibrary {
    pub fn new() -> ResourceLibrary {
        ResourceLibrary { map: BTreeMap::new() }
    }

    pub fn write_struct<'a, T: Serialize + Deserialize<'a>>(&mut self, path: String, data: &T) -> Result<()> {
        let bytes = postcard::to_allocvec(data)?;

        self.map.insert(verify_string(path)?, bytes.into_boxed_slice());

        Ok(())
    }

    pub fn read_struct<'a, T: Serialize + Deserialize<'a>>(&'a self, path: &str) -> Result<T> {
        let bytes = self.map.get(verify_str(path)?).ok_or(anyhow!("No resource exists at path '{}'.", path))?;

        let struct_: T = postcard::from_bytes(&bytes)?;

        Ok(struct_)
    }

    pub fn write_data(&mut self, path: String, data: Box<[u8]>) -> Result<()> {
        self.map.insert(verify_string(path)?, data);

        Ok(())
    }

    pub fn read_data<'a>(&'a self, path: &str) -> Result<&'a [u8]> {
        self.map.get(verify_str(path)?).ok_or(anyhow!("No resource exists at path '{}'", path)).map(|data| &data[..])
    }

    pub fn take_data(&mut self, path: &str) -> Result<Box<[u8]>> {
        self.map.remove(path).ok_or(anyhow!("No resource exists at path '{}'", path))
    }

    pub fn write_to_file(self, mut file: File) -> Result<()> {
        // Create buffers
        let mut index = Vec::new();
        let mut data_vec = Vec::new();

        // Since map is a tree map, iterator will be in order, sorted by filename
        for (filename, data) in self.map.into_iter() {
            let f_struct = FileData { filename, encrypted: false, data };
            let f_data = postcard::to_allocvec(&f_struct)?;

            // Write the current number of bytes in the buffer to our index
            let slice_tuple = (data_vec.len() as u64, f_data.len() as u64);
            index.push(slice_tuple);

            // Write to the data buffer
            data_vec.extend(f_data.into_iter());
        }

        // Convert buffers to boxed slices
        let index = index.into_boxed_slice();
        let data = data_vec.into_boxed_slice();

        // Create the struct from the buffers
        let lib_file_struct = LibraryFile { index, data };

        let lib_file_data = postcard::to_allocvec(&lib_file_struct)?;

        // Write header
        file.write(&HEADER_BYTES)?;

        // Write data
        file.write(&lib_file_data)?;

        Ok(())
    }

    pub fn read_from_file(mut file: File) -> Result<ResourceLibrary> {
        let mut first_10 = [0u8; 10];
        file.read(&mut first_10)?;

        if first_10 != HEADER_BYTES {
            bail!("File header does not match!");
        }

        let mut bytes = Vec::new();
        file.read_to_end(&mut bytes)?;

        let lib_file_struct: LibraryFile = postcard::from_bytes(&bytes)?;

        let mut file_data = Vec::new();
        file_data.reserve(lib_file_struct.index.len());
        for (pointer, size) in lib_file_struct.index.iter() {
            let data = &lib_file_struct.data[*pointer as usize..*pointer as usize + *size as usize];
            let struct_: FileData = postcard::from_bytes(data)?;

            file_data.push(struct_);
        }

        let mut out_lib = ResourceLibrary::new();
        file_data.into_iter().try_for_each(|struct_| out_lib.write_data(struct_.filename, struct_.data))?;

        Ok(out_lib)
    }
}

#[derive(Serialize, Deserialize)]
struct FileData {
    filename: String,
    encrypted: bool,
    data: Box<[u8]>
}

#[derive(Serialize, Deserialize)]
struct LibraryFile {
    index: Box<[(u64, u64)]>,
    data: Box<[u8]>
}