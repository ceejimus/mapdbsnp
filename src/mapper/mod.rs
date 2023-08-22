use std::path::Path;

use csv::{ReaderBuilder, StringRecord, WriterBuilder};

use crate::binfmt::{get_map_seek_index, rsid_to_u32, u8_to_chrom, ReadAt};

pub fn map_to_loci<P: AsRef<Path>, R: ReadAt>(
    src_tsv: &P,
    map_rdr: &mut R,
    out_path: &P,
) -> anyhow::Result<()> {
    let mut tsv_rdr = ReaderBuilder::new()
        .delimiter(b'\t')
        .has_headers(false)
        .from_path(src_tsv)?;

    let mut tsv_wtr = WriterBuilder::new()
        .delimiter(b'\t')
        .has_headers(false)
        .from_path(out_path)?;

    let num_keys_in_map = map_rdr.read_u64_at(0)?;
    let max_iters = (num_keys_in_map as f64).log2().ceil() as usize;

    // use binary search to search to find records
    for record in tsv_rdr.records() {
        let mut start = 0;
        let mut end = num_keys_in_map - 1;

        let record = record?;
        let mut record_iter = record.iter();
        let rsid = rsid_to_u32(record_iter.next().unwrap())?; // panicking on empty lines is fine with me

        for _ in 0..max_iters {
            if end < start {
                // TODO: handle this
                panic!("{} not found in map", rsid);
            }

            let middle = (end + start) / 2;
            let seek_idx = get_map_seek_index(middle);

            match map_rdr.read_u32_at(seek_idx)?.cmp(&rsid) {
                std::cmp::Ordering::Less => start = middle + 1,
                std::cmp::Ordering::Greater => end = middle - 1,
                std::cmp::Ordering::Equal => {
                    let chrom = u8_to_chrom(map_rdr.read_u8_at(seek_idx + 4)?)?;
                    let pos = map_rdr.read_u32_at(seek_idx + 4 + 1)?;
                    let loci = format!("{}:{}", chrom, pos);
                    let mut new_record = StringRecord::new();
                    new_record.push_field(&loci);
                    for field in record_iter {
                        new_record.push_field(field);
                    }
                    tsv_wtr.write_record(new_record.into_iter())?;
                    break;
                }
            }
        }
    }

    Ok(())
}
