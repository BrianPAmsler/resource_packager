pub mod resource_library;
mod index_serialization;

#[cfg(test)]
mod tests {
    use std::{fs::{File, OpenOptions}, io::Write, sync::Mutex};

    use resource_library::Result;
    use serde::Serialize;
    

    use crate::resource_library::{CompressionLevel, ResourceLibraryReader};

    use self::{index_serialization::{index_from_bytes, IndexSerializer}, resource_library::{ByteStream, ResourceLibraryWriter}};

    use super::*;

    static FILE_LOCK: Mutex<()> = Mutex::new(());

    #[test]
    fn serialization() -> Result<()> {
        let index = vec![
            ("test/a.txt".to_owned(), 0u64, 68u64),
            ("test/b.txt".to_owned(), 68, 68),
            ("test/c.txt".to_owned(), 136, 72),
            ("test/testfile.png".to_owned(), 208, 5761572)
        ].into_boxed_slice();

        let mut serializer = IndexSerializer::new();
        index.serialize(&mut serializer)?;
        let data = serializer.take();

        let deserialized_index = index_from_bytes(&data)?;

        assert_eq!(&index, &deserialized_index);

        Ok(())
    }

    #[test]
    fn invalid_path() -> Result<()> {
        let path = "test/abc?/def";

        let data = ByteStream::from([0, 1, 2, 3, 4, 5].to_vec());

        let mut lib = ResourceLibraryWriter::new();
        lib.write_stream(path.to_owned(), data).expect_err("Path should be inalid!");

        Ok(())
    }

    #[test]
    fn test_file_read_write() -> Result<()> {
        let _guard = FILE_LOCK.lock().unwrap();
        let mut lib1 = ResourceLibraryWriter::new();

        let a = ByteStream::from("Test file A".bytes().collect::<Vec<u8>>());
        let b = ByteStream::from("Test file B ".bytes().collect::<Vec<u8>>());
        let c = ByteStream::from("Test file C  ".bytes().collect::<Vec<u8>>());

        lib1.write_stream("test/a.txt".to_owned(), a)?;
        lib1.write_stream("test/b.txt".to_owned(), b)?;
        lib1.write_stream("test/c.txt".to_owned(), c)?;

        println!("Writing data...");
        let file = OpenOptions::new()
            .read(true)
            .write(true)
            .create(true)
            .truncate(true)
            .open("test/test.rcslib")?;
        lib1.write_to_file(file, CompressionLevel::Fast)?;

        println!("Reading data...");
        let lib2 = ResourceLibraryReader::new("test/test.rcslib")?;

        let debug1 = format!("{:?}", lib1.get_all_files());
        let debug2 = format!("{:?}", lib2.get_all_files());

        assert_eq!(debug1, debug2);

        Ok(())
    }

    #[test]
    fn test_file_stream() -> Result<()> {
        let _guard = FILE_LOCK.lock().unwrap();
        let testfile = File::open("test/testfile.png").expect("Please add testfile.png to test folder.");

        let mut lib1 = ResourceLibraryWriter::new();

        let a = ByteStream::from("Test file A".bytes().collect::<Vec<u8>>());
        let b = ByteStream::from("Test file B ".bytes().collect::<Vec<u8>>());
        let c = ByteStream::from("Test file C  ".bytes().collect::<Vec<u8>>());

        lib1.write_stream("test/a.txt".to_owned(), a)?;
        lib1.write_stream("test/b.txt".to_owned(), b)?;
        lib1.write_stream("test/c.txt".to_owned(), c)?;
        lib1.write_stream("test/testfile.png".to_owned(), testfile)?;

        println!("Writing file...");
        let file = OpenOptions::new()
            .read(true)
            .write(true)
            .create(true)
            .truncate(true)
            .open("test/test.rcslib")?;
        lib1.write_to_file(file, CompressionLevel::Ultra)?;

        println!("Reading File...");
        let mut reader = ResourceLibraryReader::new("test/test.rcslib")?;
        let data = reader.read_file("test/b.txt")?;

        println!("output data: '{}'", std::str::from_utf8(&data).unwrap());

        assert_eq!(lib1.read_data("test/b.txt")?, data);

        let mut testfile2 =
            OpenOptions::new()
            .read(true)
            .write(true)
            .create(true)
            .truncate(true)
            .open("test/testfile2.png")?;

        let read_file = reader.read_file("test/testfile.png")?;
        testfile2.write(&read_file)?;

        Ok(())
    }
}
