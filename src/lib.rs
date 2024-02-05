pub fn add(left: usize, right: usize) -> usize {
    left + right
}

pub mod resource_library;

#[cfg(test)]
mod tests {
    use std::{fs::{File, OpenOptions}, io::Write};

    use anyhow::Result;
    use serde::{Deserialize, Serialize};

    use crate::resource_library::{CompressionLevel, ResourceLibraryReader};

    use self::resource_library::ResourceLibrary;

    use super::*;

    #[test]
    fn read_write_u8() -> Result<()> {
        let path1 = "test/abc/def";
        let path2 = "test/abc/defg";

        let data1 = [0, 1, 2, 3, 4, 5];
        let data2 = [5, 4, 3, 2, 1, 0];

        let mut lib = ResourceLibrary::new();
        lib.write_data(path1.to_owned(), data1.to_vec().into_boxed_slice())?;
        lib.write_data(path2.to_owned(), data2.to_vec().into_boxed_slice())?;

        let read1 = lib.read_data(path1)?;
        let read2 = lib.read_data(path2)?;

        println!("{:?}", &read1[..]);
        println!("{:?}", &read2[..]);

        assert_eq!(&data1, &read1[..]);
        assert_eq!(&data2, &read2[..]);

        Ok(())
    }

    #[test]
    fn invalid_path() -> Result<()> {
        let path = "test/abc?/def";

        let data = [0, 1, 2, 3, 4, 5];

        let mut lib = ResourceLibrary::new();
        lib.write_data(path.to_owned(), data.to_vec().into_boxed_slice()).expect_err("Path should be inalid!");

        Ok(())
    }

    #[test]
    fn test_file_read_write() -> Result<()> {
        let mut lib1 = ResourceLibrary::new();
        lib1.write_data("test/a.txt".to_owned(), "Test file A".bytes().collect())?;
        lib1.write_data("test/b.txt".to_owned(), "Test file B ".bytes().collect())?;
        lib1.write_data("test/c.txt".to_owned(), "Test file C  ".bytes().collect())?;

        println!("Writing data...");
        let file = OpenOptions::new()
            .read(true)
            .write(true)
            .create(true)
            .truncate(true)
            .open("test/test.rcslib")?;
        lib1.write_to_file(file, CompressionLevel::Fast)?;

        println!("Reading data...");
        let file = File::open("test/test.rcslib")?;
        let lib2 = ResourceLibrary::read_from_file(file)?;

        let debug1 = format!("{:?}", lib1);
        let debug2 = format!("{:?}", lib2);

        assert_eq!(debug1, debug2);

        Ok(())
    }

    #[test]
    fn test_file_reader() -> Result<()> {
        let mut lib1 = ResourceLibrary::new();
        lib1.write_data("test/a.txt".to_owned(), "Test file A".bytes().collect())?;
        lib1.write_data("test/b.txt".to_owned(), "Test file B ".bytes().collect())?;
        lib1.write_data("test/c.txt".to_owned(), "Test file C  ".bytes().collect())?;

        println!("Writing data...");
        let file = OpenOptions::new()
            .read(true)
            .write(true)
            .create(true)
            .truncate(true)
            .open("test/test.rcslib")?;
        lib1.write_to_file(file, CompressionLevel::Ultra)?;

        let mut reader = ResourceLibraryReader::new("test/test.rcslib")?;
        let data = reader.read_file("test/b.txt")?;

        println!("output data: '{}'", std::str::from_utf8(&data).unwrap());

        assert_eq!(lib1.read_data("test/b.txt")?, data);

        Ok(())
    }

    #[test]
    fn test_file_stream() -> Result<()> {
        let testfile = File::open("test/testfile.png")?;

        let mut lib1 = ResourceLibrary::new();
        lib1.write_data("test/a.txt".to_owned(), "Test file A".bytes().collect())?;
        lib1.write_data("test/b.txt".to_owned(), "Test file B ".bytes().collect())?;
        lib1.write_stream("test/testfile.png".to_owned(), testfile)?;
        lib1.write_data("test/c.txt".to_owned(), "Test file C  ".bytes().collect())?;

        println!("Writing data...");
        let file = OpenOptions::new()
            .read(true)
            .write(true)
            .create(true)
            .truncate(true)
            .open("test/test.rcslib")?;
        lib1.write_to_file(file, CompressionLevel::Ultra)?;

        let mut reader = ResourceLibraryReader::new("test/test.rcslib")?;
        let data = reader.read_file("test/b.txt")?;

        println!("output data: '{}'", std::str::from_utf8(&data).unwrap());

        assert_eq!(lib1.read_data("test/b.txt")?, data);

        let mut testfile2 =
            OpenOptions::new()
            .read(true)
            .write(true)
            .create(true)
            .open("test/testfile2.png")?;

        let read_file = reader.read_file("test/testfile.png")?;
        testfile2.write(&read_file)?;

        Ok(())
    }
}
