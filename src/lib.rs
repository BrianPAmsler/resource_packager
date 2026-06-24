#[cfg(any(feature = "read", feature = "write"))]
pub mod packager;
#[cfg(any(feature = "read", feature = "write"))]
mod serialization;
#[cfg(feature = "read")]
mod peekable_stream;

#[cfg(test)]
#[cfg(any(feature = "read", feature = "write"))]
mod tests {
    use std::io::Cursor;

    use packager::Result;
    
    #[cfg(all(feature = "compression", feature = "write"))]
    use crate::packager::write::CompressionLevel;

    #[cfg(any(feature = "read", feature = "write"))]
    mod read_write {
        use std::fmt::Debug;

        use base64::{Engine, prelude::BASE64_STANDARD};

        #[cfg(any(feature = "read", all(feature = "write", feature = "compression")))]
        pub const ENCODED_FILES_COMPRESSED:   &[u8] = &base64_literal::base64_literal!("Z9dwOlQ92/UXlQAAAAAAAAADAAAAAAAAAAp0ZXN0L2EudHh0AAAAAAAAAAAAAAAAAAAACwAAAAAAAAAKdGVzdC9iLnR4dAAAAAAAAAAMAAAAAAAAAAwAAAAAAAAACnRlc3QvYy50eHQAAAAAAAAAGQAAAAAAAACMAQAAAAAAAACmAFRlc3QgZmlsZSBBAFRlc3QgZmlsZSBCIAH9N3pYWgAABObWtEYCACEBHAAAABDPWMzgAxgASV0AKhlKZ2ybRfaLaHtHl5KZKeC3qNnJaybt7quaW88/i7QB9WqSb0/fFpxBC9f4biN+QGdqBcnTLIVePzFhsOAVJQWCjQqcu38AAAAAAADwJk39Hl/KjwABZZkGAAAAiVARYrHEZ/sCAAAAAARZWg==");
        pub const ENCODED_FILES_UNCOMPRESSED: &[u8] = &base64_literal::base64_literal!("Z9dwOlQ92/UXlQAAAAAAAAADAAAAAAAAAAp0ZXN0L2EudHh0AAAAAAAAAAAAAAAAAAAACwAAAAAAAAAKdGVzdC9iLnR4dAAAAAAAAAALAAAAAAAAAAwAAAAAAAAACnRlc3QvYy50eHQAAAAAAAAAFwAAAAAAAAMZAAAAAAAAAAMwVGVzdCBmaWxlIEFUZXN0IGZpbGUgQiBUZXN0IGZpbGUgQyAgYWJjZGVmZ2hpamtsbW5vcHFyc3R1dnd4eXpBQkNERUZHSElKS0xNTk9QUVJTVFVWV1hZWmFiY2RlZmdoaWprbG1ub3BxcnN0dXZ3eHl6QUJDREVGR0hJSktMTU5PUFFSU1RVVldYWVphYmNkZWZnaGlqa2xtbm9wcXJzdHV2d3h5ekFCQ0RFRkdISUpLTE1OT1BRUlNUVVZXWFlaYWJjZGVmZ2hpamtsbW5vcHFyc3R1dnd4eXpBQkNERUZHSElKS0xNTk9QUVJTVFVWV1hZWmFiY2RlZmdoaWprbG1ub3BxcnN0dXZ3eHl6QUJDREVGR0hJSktMTU5PUFFSU1RVVldYWVphYmNkZWZnaGlqa2xtbm9wcXJzdHV2d3h5ekFCQ0RFRkdISUpLTE1OT1BRUlNUVVZXWFlaYWJjZGVmZ2hpamtsbW5vcHFyc3R1dnd4eXpBQkNERUZHSElKS0xNTk9QUVJTVFVWV1hZWmFiY2RlZmdoaWprbG1ub3BxcnN0dXZ3eHl6QUJDREVGR0hJSktMTU5PUFFSU1RVVldYWVphYmNkZWZnaGlqa2xtbm9wcXJzdHV2d3h5ekFCQ0RFRkdISUpLTE1OT1BRUlNUVVZXWFlaYWJjZGVmZ2hpamtsbW5vcHFyc3R1dnd4eXpBQkNERUZHSElKS0xNTk9QUVJTVFVWV1hZWmFiY2RlZmdoaWprbG1ub3BxcnN0dXZ3eHl6QUJDREVGR0hJSktMTU5PUFFSU1RVVldYWVphYmNkZWZnaGlqa2xtbm9wcXJzdHV2d3h5ekFCQ0RFRkdISUpLTE1OT1BRUlNUVVZXWFlaYWJjZGVmZ2hpamtsbW5vcHFyc3R1dnd4eXpBQkNERUZHSElKS0xNTk9QUVJTVFVWV1hZWmFiY2RlZmdoaWprbG1ub3BxcnN0dXZ3eHl6QUJDREVGR0hJSktMTU5PUFFSU1RVVldYWVphYmNkZWZnaGlqa2xtbm9wcXJzdHV2d3h5ekFCQ0RFRkdISUpLTE1OT1BRUlNUVVZXWFla");

        pub const FILES: &[(&str, &str)] = &[
            ("test/a.txt", "Test file A"),
            ("test/b.txt", "Test file B "),
            // Add some redundancy so the compressed and uncompressed versions are different
            ("test/c.txt", "Test file C  abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ"),
        ];

        #[derive(PartialEq, Eq)]
        pub struct Base64<'a>(pub &'a [u8]);

        impl Debug for Base64<'_> {
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                let encoded = BASE64_STANDARD.encode(self.0);
                write!(f, "base64:{encoded}")
            }
        }
    }

    use super::*;

    #[test]
    #[cfg(feature = "write")]
    fn invalid_path() -> Result<()> {
        use crate::packager::{PathError, ResourcePackagerError, write::ResourcePackageWriter};

        let path = "test/abc?/def";

        let data = vec![0, 1, 2, 3, 4, 5];
        let data = Cursor::new(data);

        let mut lib = ResourcePackageWriter::new();
        let result = lib.add_file(path.to_owned(), data);
        assert!(matches!(result, Err(ResourcePackagerError::PathError(PathError::DisallowedCharacter('?')))));

        Ok(())
    }

    #[test]
    #[cfg(feature = "write")]
    fn non_utf8_path() -> Result<()> {
        use std::path::PathBuf;
        use std::ffi::OsStr;

        use crate::packager::{PathError, ResourcePackagerError};
        use crate::packager::write::ResourcePackageWriter;

        let path = PathBuf::from(unsafe {OsStr::from_encoded_bytes_unchecked(b"\xff")});

        let data = vec![0, 1, 2, 3, 4, 5];
        let data = Cursor::new(data);

        let mut lib = ResourcePackageWriter::new();
        
        let result = lib.add_file(path.to_owned(), data);
        assert!(matches!(result, Err(ResourcePackagerError::PathError(PathError::NonUtf8Path))));

        Ok(())
    }

    #[test]
    #[cfg(feature = "write")]
    #[cfg(feature = "read")]
    fn test_file_read_write() -> Result<()> {
        use std::io::Seek;

        use crate::{packager::{read::ResourcePackageReader, write::ResourcePackageWriter}, tests::read_write::FILES};

        let mut lib1 = ResourcePackageWriter::new();

        for (path, data) in FILES {
            lib1.add_file(path, data.as_bytes())?;
        }

        println!("Writing data...");
        let mut data = Cursor::new(Vec::new());
        lib1.finish(&mut data, #[cfg(feature = "compression")] CompressionLevel::Fast)?;
        data.rewind()?;

        println!("Reading data...");
        let mut lib2 = ResourcePackageReader::new(data)?;

        let files_read: Vec<_> = lib2.get_all_files().into_iter().map(|path| path.to_owned()).collect();
        let files_read = files_read.into_iter()
            .map(|path| {
                let data = lib2.read_file(&path).unwrap();
                (path.to_str().unwrap().to_owned(), String::from_utf8(data.to_vec()).unwrap())
            })
            .collect::<Vec<_>>();
        let files_read = files_read.iter()
            .map(|(path, data)| (&path[..], &data[..]))
            .collect::<Vec<_>>();

        assert_eq!(&FILES[..], &files_read);

        Ok(())
    }

    #[test]
    #[cfg(feature = "write")]
    fn test_file_write() -> Result<()> {
        use crate::{packager::write::ResourcePackageWriter, tests::read_write::{ENCODED_FILES_UNCOMPRESSED, FILES, Base64}};

        #[cfg(feature = "compression")]
        {
            use crate::tests::read_write::{ENCODED_FILES_COMPRESSED, FILES, Base64};

            let mut lib = ResourcePackageWriter::new();

            for (path, data) in FILES {
                lib.add_file(path, data.as_bytes())?;
            }

            println!("Writing data...");
            let mut data = Cursor::new(Vec::new());
            lib.finish(&mut data, CompressionLevel::Ultra)?;
            
            let data = data.into_inner();

            assert_eq!(Base64(&data[..]), Base64(ENCODED_FILES_COMPRESSED));
        }

        let mut lib = ResourcePackageWriter::new();

        for (path, data) in FILES {
            lib.add_file(path, data.as_bytes())?;
        }

        println!("Writing data...");

        let mut data = Cursor::new(Vec::new());
        lib.finish(&mut data, #[cfg(feature = "compression")] CompressionLevel::None)?;
        
        let data = data.into_inner();

        assert_eq!(Base64(&data[..]), Base64(ENCODED_FILES_UNCOMPRESSED));

        Ok(())
    }

    #[test]
    #[cfg(feature = "read")]
    fn test_file_read() -> Result<()> {
        use crate::{packager::read::ResourcePackageReader, tests::read_write::{ENCODED_FILES_COMPRESSED, ENCODED_FILES_UNCOMPRESSED, FILES}};

        println!("Reading data...");
        let data = Cursor::new(ENCODED_FILES_COMPRESSED.to_vec());
        let lib = ResourcePackageReader::new(data);

        #[cfg(feature = "compression")]
        {
            use crate::tests::read_write::FILES;

            let mut lib = lib?;
            let files_read: Vec<_> = lib.get_all_files().into_iter().map(|path| path.to_owned()).collect();
            let files_read = files_read.into_iter()
                .map(|path| {
                    let data = lib.read_file(&path).unwrap();
                    (path.to_str().unwrap().to_owned(), String::from_utf8(data.to_vec()).unwrap())
                })
                .collect::<Vec<_>>();
            let files_read = files_read.iter()
                .map(|(path, data)| (&path[..], &data[..]))
                .collect::<Vec<_>>();

            assert_eq!(&FILES[..], &files_read);
        }
        #[cfg(not(feature = "compression"))]
        {
            use crate::packager::{CompressionNotEnabled, ResourcePackagerError};

            assert!(matches!(lib, Err(ResourcePackagerError::CompressionError(CompressionNotEnabled))))
        }
        
        println!("Reading data...");
        let data = Cursor::new(ENCODED_FILES_UNCOMPRESSED.to_vec());
        let mut lib = ResourcePackageReader::new(data)?;
        let files_read: Vec<_> = lib.get_all_files().into_iter().map(|path| path.to_owned()).collect();
        let files_read = files_read.into_iter()
            .map(|path| {
                let data = lib.read_file(&path).unwrap();
                (path.to_str().unwrap().to_owned(), String::from_utf8(data.to_vec()).unwrap())
            })
            .collect::<Vec<_>>();
        let files_read = files_read.iter()
            .map(|(path, data)| (&path[..], &data[..]))
            .collect::<Vec<_>>();

        assert_eq!(&FILES[..], &files_read);

        Ok(())
    }
}
