use std::{fmt::Debug, path::PathBuf};

use serde::{Deserialize, Serialize};
use thiserror::Error;

use crate::serialization::SerializerError;

const HEADER_BYTES: &[u8] = &[0x67, 0xD7, 0x70, 0x3A, 0x54, 0x3D, 0xDB, 0xF5, 0x17, 0x95]; // This is just a string of random numbers, it has no real signifigance
const HEADER_LEN: usize = HEADER_BYTES.len();

pub type Result<T> = std::result::Result<T, ResourcePackagerError>;

#[derive(Serialize, Deserialize, PartialEq, Eq, Debug)]
struct FileHeader {
    header_bytes: [u8; HEADER_LEN],
    index: Vec<IndexEntry>,
    use_compression: bool,
    data_len: u64
}

impl FileHeader {
    #[cfg(feature = "write")]
    pub fn new(index: Vec<IndexEntry>, data_len: u64, use_compression: bool) -> FileHeader {
        FileHeader { header_bytes: HEADER_BYTES.try_into().unwrap(), data_len, index, use_compression }
    }
}

#[derive(Serialize, Deserialize, PartialEq, Eq, Debug)]
pub struct IndexEntry {
    path: PathBuf,
    offset: u64,
    len: u64
}

#[derive(Error, Debug)]
pub enum PathError {
    #[error("Character '{0}' not allowed in path.")]
    DisallowedCharacter(char),
    #[error("No resource exists at path: {0}")]
    InvalidPath(PathBuf),
    #[error("Non-utf8 path.")]
    NonUtf8Path
}

#[cfg(not(feature = "compression"))]
#[derive(Error, Debug)]
#[error("Cannot read compressed file without compression feature enabled.")]
pub struct CompressionNotEnabled;

#[cfg(feature = "compression")]
type CompressionError = xz2::stream::Error;

#[cfg(not(feature = "compression"))]
type CompressionError = CompressionNotEnabled;

#[derive(Error, Debug)]
#[error(transparent)]
pub enum ResourcePackagerError {
    SerializationError(#[from] SerializerError),
    PathError(#[from] PathError),
    #[error("File header does not match!")]
    FileHeaderError,
    IoError(#[from] std::io::Error),
    CompressionError(#[from] CompressionError)
}

#[cfg(feature = "write")]
pub mod write {
    const FORBIDDEN_CHARACTERS: &'static str = "?%*:|\"<>,;=";

    use std::{collections::BTreeMap, fmt::Debug, io::{Read, Seek, Write}, path::{Path, PathBuf}};

    #[cfg(feature = "compression")]
    use xz2::write::XzEncoder;

    use crate::{packager::{FileHeader, IndexEntry, PathError, Result}, serialization::{self}};

    #[derive(Clone, Copy)]
    #[cfg(feature = "compression")]
    #[cfg_attr(feature = "build-binary", derive(clap::ValueEnum))]
    pub enum CompressionLevel {
        None = 99,
        Fastest = 1,
        Fast = 3,
        Normal = 5,
        Maximum = 7,
        Ultra = 9,
    }

    fn verify_path(path: &Path) -> Result<()> {
        let str = path.to_str().ok_or(PathError::NonUtf8Path)?;
        for c in str.chars() {
            for forbidden in FORBIDDEN_CHARACTERS.chars() {
                if c == forbidden {
                    return Err(PathError::DisallowedCharacter(c).into());
                }
            }
        }

        Ok(())
    }

    trait Stream: Read + Debug {}
    impl<T: Read + Debug> Stream for T {}

    #[derive(Clone, Copy, Debug)]
    pub enum Progress {
        Encoding {
            complete: u32,
            total: u32
        },
        Writing {
            written: u64,
            total: u64
        }
    }

    #[derive(Debug)]
    pub struct ResourcePackageWriter {
        map: BTreeMap<PathBuf, Box<dyn Stream>>
    }

    impl ResourcePackageWriter {
        pub fn new() -> ResourcePackageWriter {
            ResourcePackageWriter { map: BTreeMap::new() }
        }

        pub fn add_file<R: Read + Debug + 'static, P: Into<PathBuf>>(&mut self, path: P, file: R) -> Result<()> {
            let path = path.into();
            verify_path(&path)?;

            self.map.insert(path, Box::new(file));

            Ok(())
        }

        pub fn finish_with_progress<'a, W: Write + Seek>(self, mut file: W, #[cfg(feature = "compression")] compression_level: CompressionLevel, mut progress_tracker: impl FnMut(Progress)) -> Result<()> {
            let total = self.map.len() as u32;
            progress_tracker(Progress::Encoding { complete: 0, total });

            #[cfg(feature = "compression")]
            let use_compression = !matches!(compression_level, CompressionLevel::None);
            #[cfg(not(feature = "compression"))]
            let use_compression = false;

            // Create index buffer
            let mut index = Vec::new();
            let mut data_len = 0;
            let mut temp = tempfile::tempfile()?;
            // Since map is a tree map, iterator will be in order, sorted by filename
            for (i, (path, mut stream)) in self.map.into_iter().enumerate() {
                let mut data = Vec::new();
                stream.read_to_end(&mut data)?;

                #[allow(unused_mut)]
                let mut is_file_compressed = use_compression;
                
                #[cfg(feature = "compression")]
                let f_data = if use_compression {
                    // Compress data
                    let mut encoder = XzEncoder::new(Vec::new(), compression_level as u32);
                    encoder.write_all(&data)?;
                    let compressed = encoder.finish()?;

                    if compressed.len() < data.len() {
                        compressed
                    } else {
                        is_file_compressed = false;
                        data
                    }
                } else {
                    data
                };

                #[cfg(not(feature = "compression"))]
                let f_data = data;

                // Write the current number of bytes in the buffer to our index
                let offset = data_len;
                let len = f_data.len() as u64;

                if use_compression {
                    // Write single byte to indicate if file is compressed or not
                    temp.write_all(&[is_file_compressed as u8])?;
                    data_len += 1;
                }

                // Write to the file
                temp.write_all(&f_data)?;
                data_len += len;

                progress_tracker(Progress::Encoding { complete: i as u32 + 1, total });
                index.push(IndexEntry { path, offset, len });
            }

            progress_tracker(Progress::Writing { written: 0, total: data_len });

            let header = FileHeader::new(index, data_len, use_compression);

            let header_data = serialization::serialize(&header)?;

            // Write header
            file.write_all(&header_data)?;

            temp.rewind()?;

            let mut buffer = vec![0; 10*1024*1024];
            let mut written = 0;
            loop {
                let read = temp.read(&mut buffer)?;
                written += read as u64;

                if read == 0 {
                    break;
                }

                file.write_all(&buffer[..read])?;
                progress_tracker(Progress::Writing { written, total: data_len });
            }

            Ok(())
        }

        pub fn finish<'a, W: Write + Seek>(self, file: W, #[cfg(feature = "compression")] compression_level: CompressionLevel) -> Result<()> {
            self.finish_with_progress(file, #[cfg(feature = "compression")] compression_level, |_| {})
        }
    }
}

#[cfg(feature = "read")]
pub mod read {
    use std::{io::{Read, Seek}, path::{Path, PathBuf}};
    #[cfg(feature = "compression")]
    use std::io::Write;

    #[cfg(feature = "compression")]
    use xz2::write::XzDecoder;

    use crate::{packager::{FileHeader, HEADER_BYTES, PathError, ResourcePackagerError, Result}, serialization};

    pub struct ResourcePackageReader<R: Read + Seek> {
        reader: R,
        file_header: FileHeader,
        data_pointer: u64,
    }

    impl<R: Read + Seek> ResourcePackageReader<R> {
        pub fn new(mut reader: R) -> Result<ResourcePackageReader<R>> {
            let file_header: FileHeader = serialization::deserialize(&mut reader)?;

            if file_header.header_bytes != HEADER_BYTES {
                return Err(ResourcePackagerError::FileHeaderError.into());
            }

            #[cfg(not(feature = "compression"))]
            if file_header.use_compression {
                use super::CompressionNotEnabled;

                return Err(ResourcePackagerError::CompressionError(CompressionNotEnabled))
            }

            let data_pointer = reader.stream_position()?;

            Ok(ResourcePackageReader { reader, file_header, data_pointer, })
        }

        pub fn read_file<'a, P: AsRef<Path>>(&'a mut self, path: P) -> Result<Box<[u8]>> {
            let entry = self.file_header.index.binary_search_by(|entry| {
                entry.path.as_path().cmp(path.as_ref())
            }).map_err(|_| PathError::InvalidPath(path.as_ref().to_owned()))?;

            let entry = &self.file_header.index[entry];
            
            self.reader.seek(std::io::SeekFrom::Start(self.data_pointer + entry.offset))?;

            #[cfg(feature = "compression")]
            let is_file_compressed  = if self.file_header.use_compression {
                let mut buffer = [0];
                self.reader.read_exact(&mut buffer)?;

                buffer[0] > 0
            }  else {
                false
            };

            let mut buffer = vec![0u8; entry.len as usize];
            self.reader.read_exact(&mut buffer)?;

            #[cfg(feature = "compression")]
            let decompressed = if is_file_compressed {
                let mut decoder = XzDecoder::new(Vec::new());
                
                decoder.write_all(&buffer)?;
                decoder.finish()?
            } else {
                buffer
            };

            #[cfg(not(feature = "compression"))]
            let decompressed = buffer;
            
            Ok(decompressed.into_boxed_slice())
        }

        pub fn get_all_files(&self) -> Vec<PathBuf> {
            self.file_header.index.iter().map(|entry| entry.path.clone()).collect()
        }
    }
}

#[cfg(test)]
#[cfg(any(feature = "read", feature = "write"))]
const SERIALIZED_HEADAER: &[u8] = b"\x67\xd7\x70\x3a\x54\x3d\xdb\xf5\x17\x95\x00\x00\x00\x00\x00\x00\x00\x04\x00\x00\x00\x00\x00\x00\x00\x0a\x74\x65\x73\x74\x2f\x61\x2e\x74\x78\x74\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x44\x00\x00\x00\x00\x00\x00\x00\x0a\x74\x65\x73\x74\x2f\x62\x2e\x74\x78\x74\x00\x00\x00\x00\x00\x00\x00\x44\x00\x00\x00\x00\x00\x00\x00\x44\x00\x00\x00\x00\x00\x00\x00\x0a\x74\x65\x73\x74\x2f\x63\x2e\x74\x78\x74\x00\x00\x00\x00\x00\x00\x00\x88\x00\x00\x00\x00\x00\x00\x00\x48\x00\x00\x00\x00\x00\x00\x00\x11\x74\x65\x73\x74\x2f\x74\x65\x73\x74\x66\x69\x6c\x65\x2e\x70\x6e\x67\x00\x00\x00\x00\x00\x00\x00\xd0\x00\x00\x00\x00\x00\x57\xea\x24\x01\x00\x00\x00\x00\x00\x00\x30\x39";

#[test]
#[cfg(test)]
#[cfg(feature = "write")]
fn serialization() -> Result<()> {
    use crate::serialization;

    let header = FileHeader {
        header_bytes: HEADER_BYTES.try_into().unwrap(),
        index: vec![
            IndexEntry { path: "test/a.txt".into(), offset: 0, len: 68 },
            IndexEntry { path: "test/b.txt".into(), offset: 68, len: 68 },
            IndexEntry { path: "test/c.txt".into(), offset: 136, len: 72 },
            IndexEntry { path: "test/testfile.png".into(), offset: 208, len: 5761572 },
        ],
        use_compression: true,
        data_len: 12345,
    };

    let data = serialization::serialize(&header)?;

    assert_eq!(&data, &SERIALIZED_HEADAER);

    Ok(())
}

#[test]
#[cfg(test)]
#[cfg(feature = "read")]
fn deserialization() -> Result<()> {
    use std::io::Cursor;

    use crate::serialization;

    let deserialized_header: FileHeader = serialization::deserialize(&mut Cursor::new(SERIALIZED_HEADAER.to_vec()))?;

    let header = FileHeader {
        header_bytes: HEADER_BYTES.try_into().unwrap(),
        index: vec![
            IndexEntry { path: "test/a.txt".into(), offset: 0, len: 68 },
            IndexEntry { path: "test/b.txt".into(), offset: 68, len: 68 },
            IndexEntry { path: "test/c.txt".into(), offset: 136, len: 72 },
            IndexEntry { path: "test/testfile.png".into(), offset: 208, len: 5761572 },
        ],
        use_compression: true,
        data_len: 12345,
    };

    assert_eq!(&header, &deserialized_header);

    Ok(())
}

#[test]
#[cfg(test)]
#[cfg(all(feature = "write", feature = "read"))]
fn serialization_and_deserialization() -> Result<()> {
    use std::io::Cursor;

    use crate::serialization;

    let header = FileHeader {
        header_bytes: HEADER_BYTES.try_into().unwrap(),
        index: vec![
            IndexEntry { path: "test/a.txt".into(), offset: 0, len: 68 },
            IndexEntry { path: "test/b.txt".into(), offset: 68, len: 68 },
            IndexEntry { path: "test/c.txt".into(), offset: 136, len: 72 },
            IndexEntry { path: "test/testfile.png".into(), offset: 208, len: 5761572 },
        ],
        use_compression: true,
        data_len: 12345,
    };

    let data = serialization::serialize(&header)?;

    let deserialized_header: FileHeader = serialization::deserialize(&mut Cursor::new(data))?;

    assert_eq!(&header, &deserialized_header);

    Ok(())
}