mod readat;

pub(crate) use crate::binfmt::readat::ReadAt;

use std::io::Write;

const RECORD_COUNTER_SIZE: u64 = 8;
const RECORD_SIZE: u64 = 4 + 1 + 4;

pub(crate) struct MapRecord {
    pub rsid: u32,
    pub chrom: u8,
    pub pos: u32,
}

impl MapRecord {
    pub fn new(rsid: &str, chrom: &str, pos: &str) -> anyhow::Result<Self> {
        let rsid = rsid_to_u32(rsid)?;
        let chrom = chrom_to_u8(chrom)?;
        let pos = pos.parse::<u32>()?;
        Ok(Self { rsid, chrom, pos })
    }
}

pub(crate) fn get_map_seek_index(record_idx: u64) -> u64 {
    RECORD_COUNTER_SIZE + (record_idx * RECORD_SIZE)
}

pub(crate) fn write_map_record(wtr: &mut impl Write, r: &MapRecord) -> anyhow::Result<()> {
    wtr.write_all(&r.rsid.to_be_bytes())?;
    wtr.write_all(&r.chrom.to_be_bytes())?;
    wtr.write_all(&r.pos.to_be_bytes())?;
    Ok(())
}

pub(crate) fn rsid_to_u32(rsid: &str) -> anyhow::Result<u32> {
    Ok(rsid.replace("rs", "").parse::<u32>()?)
}

pub(crate) fn chrom_to_u8(chrom: &str) -> anyhow::Result<u8> {
    match chrom {
        "X" => Ok(23),
        "Y" => Ok(24),
        "MT" => Ok(25),
        _ => Ok(chrom.parse::<u8>()?),
    }
}

pub(crate) fn u8_to_chrom(x: u8) -> anyhow::Result<String> {
    Ok(match x {
        1..=22 => format!("{x}"),
        23 => "X".into(),
        24 => "Y".into(),
        25 => "MT".into(),
        _ => panic!("Invalid chrom representation {}", x),
    })
}
