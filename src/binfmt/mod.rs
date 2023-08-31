pub(crate) mod maprecord;
mod readat;

pub(crate) use crate::binfmt::maprecord::MapRecord;
use crate::binfmt::maprecord::RECORD_SIZE;
pub(crate) use crate::binfmt::readat::ReadAt;
use anyhow::anyhow;

use std::{
    fs::File,
    io::{self, Write},
    path::Path,
};

pub(crate) struct MapReader {
    mapfile: File,
    pub num_records: u64,
}

impl MapReader {
    pub(crate) fn new(mapfile_path: impl AsRef<Path>) -> io::Result<Self> {
        let mut mapfile = File::open(mapfile_path)?;
        let len = mapfile.metadata()?.len();
        let num_records = mapfile.read_u64_at(len - 8)?;
        Ok(Self {
            mapfile,
            num_records,
        })
    }

    pub(crate) fn find_record(&mut self, key: u32) -> io::Result<Option<MapRecord>> {
        let max_iters = (self.num_records as f64).log2().ceil() as usize;
        let mut start = 0;
        let mut end = self.num_records - 1;

        for _ in 0..max_iters {
            if end < start {
                return Ok(None);
            }

            let middle = (end + start) / 2;
            let seek_idx = Self::get_map_seek_index(middle);

            match self.check_record_id_at(seek_idx, key)? {
                std::cmp::Ordering::Less => start = middle + 1,
                std::cmp::Ordering::Greater => end = middle - 1,
                std::cmp::Ordering::Equal => return Ok(Some(self.read_map_record_at(seek_idx)?)),
            }
        }

        Ok(None)
    }

    fn get_map_seek_index(key: u64) -> u64 {
        key * RECORD_SIZE
    }

    fn check_record_id_at(&mut self, seek_idx: u64, key: u32) -> io::Result<std::cmp::Ordering> {
        Ok(self.mapfile.read_u32_at(seek_idx)?.cmp(&key))
    }

    fn read_map_record_at(&mut self, seek_idx: u64) -> io::Result<MapRecord> {
        let mut bytes = [0u8; RECORD_SIZE as usize];
        self.mapfile.fill_buf_at(&mut bytes, seek_idx)?;
        MapRecord::from_bytes(&bytes)
    }
}

pub(crate) struct MapWriter {
    mapfile: File,
    num_records: u64,
}

impl MapWriter {
    pub(crate) fn new(mapfile_path: impl AsRef<Path>) -> io::Result<Self> {
        let mapfile = File::create(mapfile_path)?;
        let num_records = 0;
        Ok(Self {
            mapfile,
            num_records,
        })
    }

    pub(crate) fn write_map(
        &mut self,
        map_records: impl Iterator<Item = MapRecord>,
    ) -> anyhow::Result<()> {
        let mut last_rsid = 0;

        for record in map_records {
            self.write_map_record(&record)?;
            self.num_records += 1;

            // runtime check if file is sorted and panic if not
            if last_rsid > record.rsid {
                return Err(anyhow!("Make sure source map is sorted."));
            }

            last_rsid = record.rsid;
        }

        self.append_n_records()?;

        Ok(())
    }

    fn write_map_record(&mut self, r: &MapRecord) -> anyhow::Result<()> {
        let bytes = r.to_bytes()?;
        self.mapfile.write_all(&bytes)?;
        Ok(())
    }

    fn append_n_records(&mut self) -> anyhow::Result<()> {
        // Flush contents - do we need this?
        self.mapfile.flush()?;
        // Append u64 to file
        let n = self.mapfile.write(&self.num_records.to_be_bytes())?;
        // safety check
        assert_eq!(n, 8_usize);
        Ok(())
    }
}
