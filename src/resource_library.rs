use std::{collections::BTreeMap, fs::File, io::{Read, Seek, Write}, path::Path};

use anyhow::{anyhow, bail, Result};
use serde::{Deserialize, Serialize};

const FORBIDDEN_CHARACTERS: &'static str = "\\?%*:|\"<>,;=";
const HEADER_BYTES: [u8; 10] = [0x67, 0xD7, 0x70, 0x3A, 0x54, 0x3D, 0xDB, 0xF5, 0x17, 0x95]; // This is just a string of random numbers, it has no real signifigance

#[derive(Clone, Copy)]
pub enum CompressionLevel {
    Fastest = 1,
    Fast = 3,
    Normal = 5,
    Maximum = 7,
    Ultra = 9
}

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

    pub fn write_to_file<'a>(self, mut file: File, compression_level: CompressionLevel) -> Result<()> {
        // Create buffers
        let mut index = Vec::new();
        let mut data_vec = Vec::new();

        // Since map is a tree map, iterator will be in order, sorted by filename
        for (filename, data) in self.map.into_iter() {
            let f_struct = FileData { signature: Box::new([]), data };

            let f_data = postcard::to_allocvec(&f_struct)?;

            // Compress data
            let f_data = lzma::compress(&f_data, compression_level as u32)?;

            // Write the current number of bytes in the buffer to our index
            let slice_tuple = (filename.clone(), data_vec.len() as u64, f_data.len() as u64);
            index.push(slice_tuple);

            // Write to the data buffer
            data_vec.extend(f_data.into_iter());
        }

        let index_data = postcard::to_allocvec(&index)?;

        // Write header
        file.write(&HEADER_BYTES)?;

        // Write metadataa
        file.write(&index_data.len().to_be_bytes())?;
        file.write(&data_vec.len().to_be_bytes())?;

        // Write index data
        file.write(&index_data)?;

        // Write file data
        file.write(&data_vec)?;

        Ok(())
    }

    pub fn read_from_file<'a>(mut file: File) -> Result<ResourceLibrary> {
        let mut first_10 = [0u8; 10];
        file.read(&mut first_10)?;

        if first_10 != HEADER_BYTES {
            bail!("File header does not match!");
        }

        // Read metadata
        let mut index_size = [0u8; 8];
        let mut data_size = [0u8; 8];

        file.read(&mut index_size)?;
        file.read(&mut data_size)?;

        let index_size = u64::from_be_bytes(index_size);
        let data_size = u64::from_be_bytes(data_size);

        let mut index_data = vec![0u8; index_size as usize];
        let mut data = vec![0u8; data_size as usize];

        file.read(&mut index_data)?;
        file.read(&mut data)?;

        let index: Box<[(String, u64, u64)]> = postcard::from_bytes(&index_data)?;

        let mut file_data = Vec::new();
        file_data.reserve(index.len());
        for (filename, pointer, size) in index.iter() {
            let data = &data[*pointer as usize..*pointer as usize + *size as usize];
            // Decompress data
            let data = lzma::decompress(data)?;
            let struct_: FileData = postcard::from_bytes(&data)?;

            file_data.push((filename.clone(), struct_));
        }

        let mut out_lib = ResourceLibrary::new();
        file_data.into_iter().try_for_each(|(filename, struct_)| out_lib.write_data(filename, struct_.data))?;

        Ok(out_lib)
    }
}

#[derive(Serialize, Deserialize)]
struct FileData {
    signature: Box<[u8]>,
    data: Box<[u8]>
}

pub struct ResourceLibraryReader {
    file: File,
    index: Box<[(String, u64, u64)]>,
    data_pointer: u64
}

impl ResourceLibraryReader {
    pub fn new<P: AsRef<Path>>(path: P) -> Result<ResourceLibraryReader> {
        let mut file = File::open(path)?;

        let mut first_10 = [0u8; 10];
        file.read(&mut first_10)?;

        if first_10 != HEADER_BYTES {
            bail!("File header does not match!");
        }

        // Read metadata
        let mut index_size = [0u8; 8];
        let mut data_size = [0u8; 8];

        file.read(&mut index_size)?;
        file.read(&mut data_size)?;

        let index_size = u64::from_be_bytes(index_size);
        let _data_size = u64::from_be_bytes(data_size);

        let mut index_data = vec![0u8; index_size as usize];

        file.read(&mut index_data)?;

        let index: Box<[(String, u64, u64)]> = postcard::from_bytes(&index_data)?;

        let data_pointer = file.stream_position()?;

        Ok(ResourceLibraryReader { file, index, data_pointer })
    }

    pub fn read_file<'a>(&'a mut self, path: &str) -> Result<Box<[u8]>> {
        let index = self.index.binary_search_by(|(file_path, _, _)| {
            file_path[..].cmp(path)
        }).map_err(|_| anyhow!("File not found!"))?;

        let index = &self.index[index];
        
        self.file.seek(std::io::SeekFrom::Start(self.data_pointer + index.1))?;

        let mut buffer = vec![0u8; index.2 as usize];
        self.file.read(&mut buffer)?;

        let decompressed = lzma::decompress(&buffer)?;

        let file_data: FileData = postcard::from_bytes(&decompressed)?;
        
        Ok(file_data.data)
    }
}