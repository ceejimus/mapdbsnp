use mapdbsnp::index::create_map;
use mapdbsnp::mapper::map_to_loci;
use std::{env, fs::File, path::Path};

fn main() -> anyhow::Result<()> {
    let args: Vec<String> = env::args().collect();

    if args.len() < 4 {
        panic!(
            "Usage: {} ((index mapfile_out) | (map mapfile_in map_from)) outfile",
            args[0]
        )
    }

    let cmd = args[1].clone();

    if cmd == "index" {
        let input_path = File::open(&args[2])?;
        let mapfile_path = Path::new(&args[3]);
        create_map(input_path, &mapfile_path)?;
    } else if cmd == "map" {
        let input_path = Path::new(&args[2]);
        let mut mapfile = File::open(Path::new(&args[3]))?;
        let outfile = Path::new(&args[4]);
        map_to_loci(&input_path, &mut mapfile, &outfile)?;
    } else {
        panic!("Unsupported command.")
    }

    Ok(())
}
