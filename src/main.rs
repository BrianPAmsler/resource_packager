use std::{fmt::{Debug, Display}, fs::{File, OpenOptions}, io::Write, path::{Path, PathBuf}};

use resource_packager::packager::{read::ResourcePackageReader, write::{Progress, ResourcePackageWriter}};
#[cfg(feature = "compression")] 
use resource_packager::packager::write::CompressionLevel;

use crate::differed_file::DifferedFileReader;

enum ByteCount {
    Bytes(u16),
    Kilobytes(f64),
    Megabytes(f64),
    Gigabytes(f64),
    Terabytes(f64)
}

impl Display for ByteCount {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ByteCount::Bytes(n) => write!(f, "{}B", n),
            ByteCount::Kilobytes(n) => write!(f, "{:.2} KB", n),
            ByteCount::Megabytes(n) => write!(f, "{:.2} MB", n),
            ByteCount::Gigabytes(n) => write!(f, "{:.2} GB", n),
            ByteCount::Terabytes(n) => write!(f, "{:.2} TB", n),
        }
    }
}

impl Debug for ByteCount {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ByteCount::Bytes(n) => write!(f, "{}", n),
            ByteCount::Kilobytes(n) => write!(f, "{:.2}", n),
            ByteCount::Megabytes(n) => write!(f, "{:.2}", n),
            ByteCount::Gigabytes(n) => write!(f, "{:.2}", n),
            ByteCount::Terabytes(n) => write!(f, "{:.2}", n),
        }
    }
}

impl From<u64> for ByteCount {
    fn from(value: u64) -> Self {
        let mut integer = value;
        let mut divisions = 0;
        let mut fraction = 0.0;
        while integer >= 1024 && divisions < 4 {
            fraction /= 1024.0;
            let remainder = integer % 1024;
            integer = integer / 1024;

            fraction += remainder as f64 / 1024.0;

            divisions += 1;
        }

        let count = integer as f64 + fraction;

        match divisions {
            0 => Self::Bytes(integer as u16),
            1 => Self::Kilobytes(count),
            2 => Self::Megabytes(count),
            3 => Self::Gigabytes(count),
            4 => Self::Terabytes(count),
            _ => unreachable!()
        }
    }
}

fn read_dir_all(dir: &Path) -> Vec<PathBuf> {
    let mut paths = Vec::new();

    fn recursive(dir: &Path, paths: &mut Vec<PathBuf>) {
        let (dirs, files): (Vec<_>, Vec<_>) = std::fs::read_dir(dir).unwrap()
            .filter_map(|entry| entry.ok())
            .map(|entry| entry.path())
            .partition(|path| path.is_dir());

        paths.extend(files);

        for dir in dirs {
            recursive(dir.as_path(), paths);
        }
    }

    recursive(dir, &mut paths);

    paths
}

macro_rules! usage {
    () => {
        {
            eprintln!("Usage: resource-packager   pack path [file_name]\n       resource-packager unpack path [output_dir]");
            std::process::exit(0)
        }
    };
}

fn print_progress(progress: Progress) {
    // Move cursor if progress is not 0
    match &progress {
        Progress::Encoding { complete: 0, .. }
        | Progress::Writing { written: 0, .. } => (),
        _ => print!("\r\x1b[1A")
    }

    match progress {
        Progress::Encoding { complete, total } => println!("  Encoding files: [{} / {}]", complete, total),
        Progress::Writing { written, total } => {
            let total = ByteCount::from(total);
            let written = match total {
                ByteCount::Bytes(_) => ByteCount::Bytes(written as u16),
                ByteCount::Kilobytes(_) => ByteCount::Kilobytes(written as f64 / 1024.0f64.powi(1)),
                ByteCount::Megabytes(_) => ByteCount::Megabytes(written as f64 / 1024.0f64.powi(2)),
                ByteCount::Gigabytes(_) => ByteCount::Gigabytes(written as f64 / 1024.0f64.powi(3)),
                ByteCount::Terabytes(_) => ByteCount::Terabytes(written as f64 / 1024.0f64.powi(4)),
            };

            println!("  Writing files: [{:?} / {}]", written, total)
        },
    }
}

mod differed_file {
    use std::{fs::File, io::{Read, Seek}, path::PathBuf};

    enum Inner {
        File(File),
        Path(PathBuf)
    }

    pub struct DifferedFileReader {
        inner: Inner
    }

    impl DifferedFileReader {
        pub fn new(path: PathBuf) -> DifferedFileReader {
            DifferedFileReader { inner: Inner::Path(path) }
        }

        fn file(&mut self) -> std::io::Result<&mut File> {
            if let Inner::Path(path) = &self.inner {
                let file = File::open(path)?;
                self.inner = Inner::File(file);
            }

            let Inner::File(file) = &mut self.inner  else { unreachable!() };

            Ok(file)
        }
    }

    impl Read for DifferedFileReader {
        fn read(&mut self, buf: &mut [u8]) -> std::io::Result<usize> {
            let file = self.file()?;

            file.read(buf)
        }
    }

    impl Seek for DifferedFileReader {
        fn seek(&mut self, pos: std::io::SeekFrom) -> std::io::Result<u64> {
            let file = self.file()?;

            file.seek(pos)
        }
    }
}

fn main() {
    enable_ansi_support::enable_ansi_support().unwrap();

    let args: Vec<String> = std::env::args().collect();
    let (args, options): (Vec<_>, Vec<_>) = args.iter().map(|arg| &arg[..])
        .partition(|arg| !arg.starts_with('-'));
    let args = match args.as_slice() {
        [_, command, path] => [*command, *path, ""],
        [_, command, path, name] => [*command, *path, *name],
        _ => usage!()
    };

    #[cfg(feature = "compression")] 
    let mut compressed = false;
    for option in options {
        match option {
            #[cfg(feature = "compression")] 
            "-c" | "-compress" => {
                if compressed {
                    eprintln!("Duplicate option: 'compress'");
                    std::process::exit(0);
                }

                compressed = true;
            },
            _ => {
                eprintln!("Unknown option: '{}'", &option[1..]);
                std::process::exit(0);
            }
        }
    }

    match args {
        ["pack", dir, name] => {
            let Ok(dir) = Path::new(dir).canonicalize() else {
                eprintln!("Error: resource_directory must be a valid directory.");
                return;
            };

            if !dir.exists() || !dir.is_dir() {
                eprintln!("Error: resource_directory must be a valid directory.");
                return;
            }

            let name = if name == "" {
                PathBuf::from(dir.file_name().unwrap().to_str().unwrap()).with_added_extension("pack")
            } else {
                name.into()
            };

            let todo = (); // TODO: Add compression option in cli
            println!("Packing files:");
            println!("  Scanning directory...");
            let mut pack = ResourcePackageWriter::new();

            read_dir_all(dir.as_path())
                .into_iter()
                .map(|file| (file.strip_prefix(&dir).unwrap().to_owned(), file))
                .for_each(|(path, absolute_path)| {
                    let file = File::open(absolute_path).unwrap();
                    pack.add_file(path, file).unwrap();
                });
            
            let out = OpenOptions::new()
                .write(true)
                .create(true)
                .truncate(true)
                .open(name)
                .unwrap();

            #[cfg(feature = "compression")]
            let compression = match compressed {
                true => CompressionLevel::Ultra,
                false => CompressionLevel::None,
            };

            pack.finish_with_progress(out, #[cfg(feature = "compression")] compression, print_progress).unwrap();
            println!("  Done");
        },
        ["unpack", file, output_dir] => {
            let path = Path::new(file);
            let root = if output_dir == "" {
                Path::new(path.file_prefix().unwrap())
            } else {
                Path::new(output_dir)
            };

            let file = DifferedFileReader::new(path.to_owned());
            let mut pack = ResourcePackageReader::new(file).unwrap();

            println!("Unpacking files:");
            println!("  Reading package contents...");
            let files = pack.get_all_files();
            let count = files.len();
            for (i, path) in files.into_iter().enumerate() {
                let data = pack.read_file(&path).unwrap();
                let path = {
                    let mut temp = root.to_owned();
                    temp.push(path);
                    temp
                };
                
                std::fs::create_dir_all(&path.parent().unwrap()).unwrap();
                let mut file = OpenOptions::new()
                    .write(true)
                    .create(true)
                    .truncate(true)
                    .open(&path).unwrap();
                file.write_all(&data).unwrap();

                if i != 0 {
                    print!("\r\x1b[1A");
                }

                println!("  Extracting files: [{} / {}]", i + 1, count);
            }

            println!("  Done");
        },
        _ => usage!()
    }
}