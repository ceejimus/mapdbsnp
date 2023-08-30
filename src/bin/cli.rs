use clap::Subcommand;
use clap::{Args, Parser};
use mapdbsnp::index::create_map;
use mapdbsnp::mapper::map_to_loci;
use std::{fs::File, path::PathBuf};

#[derive(Parser, Debug)]
struct Cli {
    #[clap(subcommand)]
    command: CliCommand,
}

#[derive(Subcommand, Debug)]
enum CliCommand {
    #[clap(name = "index")]
    /// Creates an dbSNP -> genomic loci map file from TSV
    Index(IndexCommand),
    #[clap(name = "map")]
    /// Maps a source TSV w/ dbSNP IDs to a genomic loci file using mapfile
    Map(MapCommand),
}

#[derive(Args, Debug)]
struct IndexCommand {
    /// Path to input TSV
    input_tsv: PathBuf,
    /// Path to mapfile output
    mapfile: PathBuf,
}

#[derive(Args, Debug)]
struct MapCommand {
    /// Path to source TSV
    source_tsv: PathBuf,
    /// Path to mapfile
    mapfile: PathBuf,
    /// Path to output TSV
    output_tsv: PathBuf,
}

fn main() -> anyhow::Result<()> {
    let cli: Cli = Parser::parse();

    match &cli.command {
        CliCommand::Index(command) => handle_index_command(command),
        CliCommand::Map(command) => handle_map_command(command),
    }
}

fn handle_index_command(command: &IndexCommand) -> anyhow::Result<()> {
    let input_file = File::open(&command.input_tsv)?;
    create_map(input_file, &command.mapfile)
}

fn handle_map_command(command: &MapCommand) -> anyhow::Result<()> {
    let mut mapfile = File::open(&command.mapfile)?;
    map_to_loci(&command.source_tsv, &mut mapfile, &command.output_tsv)
}
