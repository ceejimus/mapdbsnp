use crate::binfmt::write_map_record;
use crate::binfmt::MapRecord;
use std::{
    fs::{self, File},
    io::{self, BufWriter, Write},
    path::Path,
};

use csv::{Reader, ReaderBuilder, StringRecord};
use mktemp::Temp;

pub fn create_map<P: AsRef<Path>>(src_tsv: &P, dst: &P) -> anyhow::Result<()> {
    let mut rdr = ReaderBuilder::new()
        .delimiter(b'\t')
        .has_headers(false)
        .from_path(src_tsv)?;

    let num_records = write_map_records(dst, &mut rdr)?;
    prepend_file(&num_records.to_be_bytes(), dst)?;

    Ok(())
}

fn write_map_records<P: AsRef<Path>>(dst: &P, rdr: &mut Reader<File>) -> anyhow::Result<usize> {
    // scope of mapfile
    // we want to make sure mapfile is flushed and dropped before we prepend num_records
    let mut map_wtr = BufWriter::new(File::create(dst)?);

    // runtime check if file is sorted and panic if not
    let mut last_rsid = 0;

    let mut num_records: usize = 0;

    for r in rdr.records() {
        let r = r?;
        let record = parse_map_record(r)?;
        write_map_record(&mut map_wtr, &record)?;
        num_records += 1;

        if last_rsid > record.rsid {
            panic!("Make sure source map is sorted.")
        }

        last_rsid = record.rsid;
    }
    map_wtr.flush()?;

    Ok(num_records)
}

pub(crate) fn parse_map_record(r: StringRecord) -> anyhow::Result<MapRecord> {
    let mut parts = r[1].split(':');
    MapRecord::new(&r[0], parts.next().unwrap(), parts.next().unwrap())
}

fn prepend_file<P: AsRef<Path>>(data: &[u8], file_path: &P) -> anyhow::Result<()> {
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
