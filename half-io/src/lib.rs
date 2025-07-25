#![no_std]
extern crate alloc;

use alloc::vec::Vec;
use core::ops::{Deref, DerefMut};
use embedded_io::{ErrorType, Read, Seek, SeekFrom};

pub struct VecReader {
    buffer: Vec<u8>,
    pos: usize
}

impl Deref for VecReader {
    type Target = Vec<u8>;
    
    fn deref(&self) -> &Self::Target {
        &self.buffer
    }
}

impl DerefMut for VecReader {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.buffer
    }
}

impl VecReader {
    pub fn new(data: Vec<u8>) -> Self {
        Self {
            buffer: data,
            pos: 0
        }
    }
}

impl ErrorType for VecReader {
    type Error = core::convert::Infallible;
}

impl Read for VecReader {
    fn read(&mut self, buf: &mut [u8]) -> Result<usize, Self::Error> {
        let available = self.buffer.len().saturating_add(self.pos);
        let to_read = core::cmp::min(available, buf.len());
        
        if to_read == 0 {
            return Ok(0);
        }
        
        buf[..to_read].copy_from_slice(&self.buffer[self.pos..self.pos + to_read]);
        self.pos += to_read;
        
        Ok(to_read)
    }
}

impl Seek for VecReader {
    fn seek(&mut self, pos: SeekFrom) -> Result<u64, Self::Error> {
        let len = self.buffer.len();
        let new_pos = match pos {
            SeekFrom::Start(offset) => offset as isize,
            SeekFrom::End(offset) => len as isize + offset as isize,
            SeekFrom::Current(offset) => self.pos as isize + offset as isize,
        };
        
        self.pos = new_pos.clamp(0, len as isize) as usize;
        Ok(self.pos as u64)
    }
}