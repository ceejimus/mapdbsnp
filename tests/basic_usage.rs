use std::{
    fs::File,
    io::{self, Read},
};

use mapdbsnp::index::create_map;
use mapdbsnp::mapper::map_to_loci;
use mktemp::Temp;

fn cmp_files(path1: &str, path2: &str) -> io::Result<bool> {
    let mut f1 = File::open(path1)?;
    let mut f2 = File::open(path2)?;

    let mut b1 = Vec::new();
    let mut b2 = Vec::new();

    let _ = f1.read_to_end(&mut b1);
    let _ = f2.read_to_end(&mut b2);

    Ok(b2 == b1)
}

#[test]
fn test_make_map() {
    let dir = Temp::new_dir().unwrap();
    let dir_path = dir.to_path_buf();

    let out_map_path = dir_path.join("test.map");
    let expected_map_path = "tests/data/test.map.10.map";
    let src_tsv = File::open("tests/data/test.map.10.tsv").unwrap();

    create_map(src_tsv, &out_map_path.as_path()).unwrap();
    cmp_files(expected_map_path, out_map_path.to_str().unwrap()).unwrap();
}

#[test]
fn test_map_markers() {
    let dir = Temp::new_dir().unwrap();
    let dir_path = dir.to_path_buf();

    let out_tsv_path = dir_path.join("out.tsv");
    let expected_tsv_path = "tests/data/test.out.10.tsv";
    let src_tsv_path = "tests/data/test.in.10.tsv";
    let src_map_path = "tests/data/test.map.10.map";
    let mut mapfile = File::open(src_map_path).unwrap();

    map_to_loci(
        &src_tsv_path,
        &mut mapfile,
        &out_tsv_path.as_path().to_str().unwrap(),
    )
    .unwrap();
    cmp_files(expected_tsv_path, out_tsv_path.to_str().unwrap()).unwrap();
}
