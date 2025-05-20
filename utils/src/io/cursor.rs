use alloc::vec::Vec;
use core::cmp::min;
use core::convert::Infallible;
use core::fmt::Debug;
use embedded_io::{ErrorType, Read, Seek, SeekFrom};

pub struct ByteCursor {
    buf: Vec<u8>,
    pos: usize
}

impl ByteCursor {
    pub fn new(buf: Vec<u8>) -> Self {
        Self {
            buf, 
            pos: 0
        }
    }
}

impl ErrorType for ByteCursor { type Error = Infallible; }

impl Read for ByteCursor {
    fn read(&mut self, out: &mut [u8]) -> Result<usize, Self::Error> {
        let available = &self.buf[self.pos..];
        let len = min(out.len(), available.len());
        out[..len].copy_from_slice(&available[..len]);
        self.pos += len;
        Ok(len)
    }
}

impl Seek for ByteCursor {
    fn seek(&mut self, pos: SeekFrom) -> Result<u64, Self::Error> {
        let len = self.buf.len() as i64;
        let cur = self.pos as i64;
        
        let new_pos = match pos {
            SeekFrom::Start(pos) => pos as i64,
            SeekFrom::End(pos) => len + pos,
            SeekFrom::Current(pos) => cur + pos
        };
        
        self.pos = new_pos.clamp(0, len) as usize;
        Ok(self.pos as u64)
    }
}