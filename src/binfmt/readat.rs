use std::io::{self, Cursor, Read, Seek, SeekFrom};

use byteorder::{BigEndian, ReadBytesExt};

pub trait ReadAt {
    fn read_u8_at(&mut self, offset: u64) -> io::Result<u8>;
    fn read_u32_at(&mut self, offset: u64) -> io::Result<u32>;
    fn read_u64_at(&mut self, offset: u64) -> io::Result<u64>;
    fn fill_buf_at(&mut self, buf: &mut [u8], offset: u64) -> io::Result<()>;
}

impl<R: Read + Seek> ReadAt for R {
    fn read_u8_at(&mut self, offset: u64) -> io::Result<u8> {
        let mut buf = [0u8; 1];
        self.fill_buf_at(&mut buf, offset)?;
        Cursor::new(buf).read_u8()
    }

    fn read_u32_at(&mut self, offset: u64) -> io::Result<u32> {
        let mut buf = [0u8; 4];
        self.fill_buf_at(&mut buf, offset)?;
        Cursor::new(buf).read_u32::<BigEndian>()
    }

    fn read_u64_at(&mut self, offset: u64) -> io::Result<u64> {
        let mut buf = [0u8; 8];
        self.fill_buf_at(&mut buf, offset)?;
        Cursor::new(buf).read_u64::<BigEndian>()
    }

    fn fill_buf_at(&mut self, buf: &mut [u8], offset: u64) -> io::Result<()> {
        self.seek(SeekFrom::Start(offset))?;
        self.read_exact(buf)
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn can_read_bytes_at() {
        let data = vec![1, 2, 3, 4, 5];
        let mut cursor = Cursor::new(data);

        let mut buf = [0u8; 2];
        cursor.fill_buf_at(&mut buf, 1).unwrap();

        assert_eq!([2, 3], buf);
    }

    #[test]
    fn can_read_u8_at() {
        let data = vec![1, 2, 3, 4, 5];
        let mut cursor = Cursor::new(data);

        let u8_read = cursor.read_u8_at(2).unwrap();

        assert_eq!(3, u8_read);
    }

    #[test]
    fn can_read_u32_at() {
        let data = vec![1, 0x0, 0x72, 0x31, 0x61];
        let mut cursor = Cursor::new(data);

        let u32_read = cursor.read_u32_at(1).unwrap();

        assert_eq!(0x00723161, u32_read);
    }

    #[test]
    fn can_read_u64_at() {
        let data = vec![1, 0x0, 0x72, 0x31, 0x61, 0x05, 0x43, 0xf5, 0xc3];
        let mut cursor = Cursor::new(data);

        let u64_read = cursor.read_u64_at(1).unwrap();

        assert_eq!(0x007231610543f5c3, u64_read);
    }
}
