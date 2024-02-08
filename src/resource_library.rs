use std::{collections::BTreeMap, fmt::Debug, fs::File, io::{Read, Seek, SeekFrom, Write}, path::Path};

use anyhow::{anyhow, bail, Result};
use serde::Serialize;

use crate::index_serialization::{index_from_bytes, IndexSerializer};

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

pub struct ByteStream {
    bytes: Box<[u8]>,
    position: usize
}

impl Read for ByteStream {
    fn read(&mut self, buf: &mut [u8]) -> std::io::Result<usize> {
        let bytes_read = usize::min(buf.len(), self.bytes.len() - self.position);
        buf[..bytes_read].copy_from_slice(&self.bytes[self.position..self.position + bytes_read]);

        self.position += bytes_read;

        Ok(bytes_read)
    }
}

impl Write for ByteStream {
    fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
        let bytes_written = usize::min(buf.len(), self.bytes.len() - self.position);

        self.bytes[self.position..self.position + bytes_written].copy_from_slice(&buf[..bytes_written]);

        self.position += bytes_written;

        Ok(bytes_written)
    }

    fn flush(&mut self) -> std::io::Result<()> {
        // I don't think this needs to do anything
        Ok(())
    }
}

impl Seek for ByteStream {
    fn seek(&mut self, pos: std::io::SeekFrom) -> std::io::Result<u64> {
        match pos {
            SeekFrom::Start(offset) => self.position = offset as usize,
            SeekFrom::End(offset) => self.position = (self.bytes.len() as i64 + offset) as usize,
            SeekFrom::Current(offset) => self.position = (self.position as i64 + offset) as usize,
        }

        if self.position > self.bytes.len() {
            self.position = self.bytes.len();
        }

        Ok(self.position as u64)
    }
}

impl Debug for ByteStream {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.bytes.fmt(f)
    }
}

impl From<Box<[u8]>> for ByteStream {
    fn from(value: Box<[u8]>) -> Self {
        ByteStream { bytes: value, position: 0 }
    }
}

impl From<Vec<u8>> for ByteStream {
    fn from(value: Vec<u8>) -> Self {
        ByteStream { bytes: value.into_boxed_slice(), position: 0 }
    }
}

pub trait Resource: Read + Seek + Debug {} 
impl<T: Read + Seek + Debug> Resource for T {}

#[derive(Debug)]
pub struct ResourceLibraryWriter {
    map: BTreeMap<String, Box<dyn Resource>>
}

impl ResourceLibraryWriter {
    pub fn new() -> ResourceLibraryWriter {
        ResourceLibraryWriter { map: BTreeMap::new() }
    }

    // pub fn write_data(&mut self, path: String, data: Box<[u8]>) -> Result<()> {
    //     self.map.insert(verify_string(path)?, Box::new(ByteStream::from(data)));

    //     Ok(())
    // }

    pub fn write_stream<T: Read + Seek + Debug + 'static>(&mut self, path: String, stream: T) -> Result<()> {
        self.map.insert(verify_string(path)?, Box::new(stream));

        Ok(())
    }

    pub fn read_data<'a>(&'a mut self, path: &str) -> Result<Box<[u8]>> {
        match self.map.get_mut(verify_str(path)?).ok_or(anyhow!("No resource exists at path '{}'", path)) {
            Ok(resource) => {
                let mut bytes = Vec::new();
                resource.rewind()?;
                resource.read_to_end(&mut bytes)?;
    
                Ok(bytes.into_boxed_slice())
            },
            Err(err) => Err(err)
        }
    }

    pub fn take_data(&mut self, path: &str) -> Result<Box<[u8]>> {
        match self.map.remove(path).ok_or(anyhow!("No resource exists at path '{}'", path)) {
            Ok(mut resource) => {
                let mut bytes = Vec::new();
                resource.rewind()?;
                resource.read_to_end(&mut bytes)?;
    
                Ok(bytes.into_boxed_slice())
            },
            Err(err) => Err(err)
        }
    }

    pub fn write_to_file<'a>(&mut self, mut file: File, compression_level: CompressionLevel) -> Result<()> {
        // Create index template

        // Create index buffer
        let mut index = Vec::new();
        // Since map is a tree map, iterator will be in order, sorted by filename
        for (filename, _) in self.map.iter_mut() {
            // Write zeroes to be replaced later
            let slice_tuple = (filename.clone(), u64::MAX, u64::MAX);
            index.push(slice_tuple);
        }

        let mut serializer = IndexSerializer::new();
        index.serialize(&mut serializer)?;
        let index_data = serializer.take();

        // Write header
        file.write(&HEADER_BYTES)?;

        // Write metadataa
        println!("initial index size: {}", index_data.len());
        file.write(&index_data.len().to_be_bytes())?;

        let data_len_offset = file.stream_position()?;
        file.write(&0u64.to_be_bytes())?;

        // Write index data
        file.write(&index_data)?;

        let mut data_len = 0;

        // Since map is a tree map, iterator will be in order, sorted by filename
        for (i, (_, resource)) in self.map.iter_mut().enumerate() {
            let mut data = Vec::new();
            resource.rewind()?;
            resource.read_to_end(&mut data)?;
            let data = data.into_boxed_slice();

            // Compress data
            let f_data = lzma::compress(&data, compression_level as u32)?;

            // Write the current number of bytes in the buffer to our index
            index[i].1 = data_len;
            index[i].2 = f_data.len() as u64;

            // Write to the file
            file.write(&f_data[..])?;
            data_len += f_data.len() as u64;
        }

        // Update data length
        file.seek(SeekFrom::Start(data_len_offset))?;
        file.write(&data_len.to_be_bytes())?;

        // Update index
        let mut serializer = IndexSerializer::new();
        index.serialize(&mut serializer)?;
        let index_data = serializer.take();
        file.write(&index_data)?;

        Ok(())
    }

    // pub fn read_from_file<'a>(mut file: File) -> Result<ResourceLibrary> {
    //     let mut first_10 = [0u8; 10];
    //     file.read(&mut first_10)?;

    //     if first_10 != HEADER_BYTES {
    //         bail!("File header does not match!");
    //     }

    //     // Read metadata
    //     let mut index_size = [0u8; 8];
    //     let mut data_size = [0u8; 8];

    //     file.read(&mut index_size)?;
    //     file.read(&mut data_size)?;

    //     let index_size = u64::from_be_bytes(index_size);
    //     let data_size = u64::from_be_bytes(data_size);

    //     let mut index_data = vec![0u8; index_size as usize];
    //     let mut data = vec![0u8; data_size as usize];

    //     file.read(&mut index_data)?;
    //     file.read(&mut data)?;

    //     let reader = Reader::get_root(&index_data[..])?;
    //     let index = Box::<[(String, u64, u64)]>::deserialize(reader)?;

    //     let mut file_data = Vec::new();
    //     file_data.reserve(index.len());
    //     for (filename, pointer, size) in index.iter() {
    //         let data = &data[*pointer as usize..*pointer as usize + *size as usize];
    //         // Decompress data
    //         let data = lzma::decompress(data)?;
    //         // let struct_: FileData = postcard::from_bytes(&data)?;

    //         file_data.push((filename.clone(), struct_));
    //     }

    //     let mut out_lib = ResourceLibrary::new();
    //     file_data.into_iter().try_for_each(|(filename, struct_)| out_lib.write_data(filename, struct_.data))?;

    //     Ok(out_lib)
    // }

    pub fn get_all_files(&self) -> Box<[&str]> {
        self.map.keys().map(|path| &path[..]).collect()
    }
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

        let index = index_from_bytes(&index_data)?;

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
        
        Ok(decompressed.into_boxed_slice())
    }

    pub fn get_all_files(&self) -> Box<[&str]> {
        self.index.iter().map(|(path, _, _)| &path[..]).collect()
    }
}