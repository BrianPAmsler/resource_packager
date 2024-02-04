pub fn add(left: usize, right: usize) -> usize {
    left + right
}

pub mod resource_library;

#[cfg(test)]
mod tests {
    use std::fs::{File, OpenOptions};

    use anyhow::Result;
    use serde::{Deserialize, Serialize};

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

        assert_eq!(&data1, read1);
        assert_eq!(&data2, read2);

        Ok(())
    }

    #[test]
    fn read_write_struct() -> Result<()> {
        let path1 = "test/abc/def";
        let path2 = "test/abc/defg";

        #[derive(Serialize, Deserialize, Debug, PartialEq)]
        struct Test1 {
            a: u32,
            b: u32,
            c: u32,
            d: u32
        }

        #[derive(Serialize, Deserialize, Debug, PartialEq)]
        struct Test2 {
            a: f32,
            b: f32,
            c: f32,
            d: f32
        }

        let data1 = Test1 { a: 0, b: 1, c: 2, d: 3 };
        let data2 = Test2 { a: 3.0, b: 2.0, c: 1.0, d: 0.0 };

        let mut lib = ResourceLibrary::new();
        lib.write_struct(path1.to_owned(), &data1)?;
        lib.write_struct(path2.to_owned(), &data2)?;

        let read1 = lib.read_struct::<Test1>(path1)?;
        let read2 = lib.read_struct::<Test2>(path2)?;

        assert_eq!(&data1, &read1);
        assert_eq!(&data2, &read2);

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
            .open("test.rcslib")?;
        lib1.clone().write_to_file(file)?;

        println!("Reading data...");
        let file = File::open("test.rcslib")?;
        let lib2 = ResourceLibrary::read_from_file(file)?;

        assert_eq!(lib1, lib2);

        Ok(())
    }
}
