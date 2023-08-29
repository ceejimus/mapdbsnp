pub(crate) mod maprecord;
mod readat;

use mktemp::Temp;

pub(crate) use crate::binfmt::maprecord::MapRecord;
use crate::binfmt::maprecord::RECORD_SIZE;
pub(crate) use crate::binfmt::readat::ReadAt;

use std::{
    fs::{self, File},
    io::{self, Write},
    path::Path,
};

const RECORD_COUNTER_SIZE: u64 = 8;

pub(crate) fn get_map_seek_index(record_idx: u64) -> u64 {
    RECORD_COUNTER_SIZE + (record_idx * RECORD_SIZE)
}

pub(crate) fn write_map_record(wtr: &mut impl Write, r: &MapRecord) -> anyhow::Result<()> {
    let bytes = r.to_bytes()?;
    wtr.write_all(&bytes)?;
    Ok(())
}

pub(crate) fn check_record_id_at<R: ReadAt>(
    map_rdr: &mut R,
    seek_idx: u64,
    rsid: u32,
) -> anyhow::Result<std::cmp::Ordering> {
    Ok(map_rdr.read_u32_at(seek_idx)?.cmp(&rsid))
}

pub(crate) fn read_map_record_at<R: ReadAt>(
    map_rdr: &mut R,
    seek_idx: u64,
) -> io::Result<MapRecord> {
    let mut bytes = [0u8; RECORD_SIZE as usize];
    map_rdr.fill_buf_at(&mut bytes, seek_idx)?;
    MapRecord::from_bytes(&bytes)
}

pub(crate) fn prepend_file<P: AsRef<Path>>(data: &[u8], file_path: &P) -> anyhow::Result<()> {
    // Create a temporary file
    let tmp_path = Temp::new_file()?;
    // Open temp file for writing
    let mut tmp = File::create(&tmp_path)?;
    // Open source file for reading
    let mut src = File::open(file_path)?;
    // Write the data to prepend
    tmp.write_all(data)?;
    // Copy the rest of the source file
    io::copy(&mut src, &mut tmp)?;
    fs::remove_file(file_path)?;
    fs::rename(&tmp_path, file_path)?;
    // Stop the temp file being automatically deleted when the variable
    // is dropped, by releasing it.
    tmp_path.release();
    Ok(())
}
