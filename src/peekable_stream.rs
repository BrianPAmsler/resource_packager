#![allow(unused)]
use std::io::{Read, Seek};

pub struct Peekable<Inner: Read + Seek> {
    inner: Inner
}

impl<Inner: Read + Seek> Peekable<Inner> {
    pub fn new(inner: Inner) -> Peekable<Inner> {
        Peekable { inner }
    }

    pub fn peek(&mut self, buf: &mut [u8]) -> std::io::Result<usize> {
        let pos = self.inner.stream_position()?;
        let count = self.inner.read(buf)?;

        self.inner.seek(std::io::SeekFrom::Start(pos))?;

        Ok(count)
    }

    pub fn peek_exact(&mut self, buf: &mut [u8]) -> std::io::Result<()> {
        let pos = self.inner.stream_position()?;
        self.inner.read_exact(buf)?;

        self.inner.seek(std::io::SeekFrom::Start(pos))?;

        Ok(())
    }

    pub fn peek_to_end(&mut self, buf: &mut Vec<u8>) -> std::io::Result<usize> {
        let pos = self.inner.stream_position()?;
        let count = self.inner.read_to_end(buf)?;

        self.inner.seek(std::io::SeekFrom::Start(pos))?;

        Ok(count)
    }

    fn read_to_string(&mut self, buf: &mut String) -> std::io::Result<usize> {
        let pos = self.inner.stream_position()?;
        let count = self.inner.read_to_string(buf)?;

        self.inner.seek(std::io::SeekFrom::Start(pos))?;

        Ok(count)
    }

    fn read_vectored(&mut self, bufs: &mut [std::io::IoSliceMut<'_>]) -> std::io::Result<usize> {
        let pos = self.inner.stream_position()?;
        let count = self.inner.read_vectored(bufs)?;

        self.inner.seek(std::io::SeekFrom::Start(pos))?;

        Ok(count)
    }
}

impl<Inner: Read + Seek> Read for Peekable<Inner> {
    fn read(&mut self, buf: &mut [u8]) -> std::io::Result<usize> {
        self.inner.read(buf)
    }

    fn read_exact(&mut self, buf: &mut [u8]) -> std::io::Result<()> {
        self.inner.read_exact(buf)
    }

    fn read_to_end(&mut self, buf: &mut Vec<u8>) -> std::io::Result<usize> {
        self.inner.read_to_end(buf)
    }

    fn read_to_string(&mut self, buf: &mut String) -> std::io::Result<usize> {
        self.inner.read_to_string(buf)
    }

    fn read_vectored(&mut self, bufs: &mut [std::io::IoSliceMut<'_>]) -> std::io::Result<usize> {
        self.inner.read_vectored(bufs)
    }
}

impl<Inner: Read + Seek> Seek for Peekable<Inner> {
    fn rewind(&mut self) -> std::io::Result<()> {
        self.inner.rewind()
    }

    fn seek(&mut self, pos: std::io::SeekFrom) -> std::io::Result<u64> {
        self.inner.seek(pos)
    }

    fn seek_relative(&mut self, offset: i64) -> std::io::Result<()> {
        self.inner.seek_relative(offset)
    }

    fn stream_position(&mut self) -> std::io::Result<u64> {
        self.inner.stream_position()
    }
}