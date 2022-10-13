use clap::Parser;
use meshquisse::trianglemerger::MeshMerger;
use std::{io::Read, time::SystemTime};

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// path to the mesh to merge
    #[arg(short, long)]
    path: String,
}

fn main() {
    let args = Args::parse();

    let start = SystemTime::now();
    let mut file = std::fs::File::open(args.path).unwrap();
    let mut buffer = Vec::new();
    file.read_to_end(&mut buffer).unwrap();
    let mut mesh_merger = MeshMerger::from_bytes(&buffer);
    let end = SystemTime::now();
    let elapsed = end.duration_since(start);
    println!(
        "Reading took around {}s",
        elapsed.unwrap_or_default().as_secs_f32()
    );
    let start = SystemTime::now();
    mesh_merger.my_merge();
    mesh_merger.remove_unused();
    // TODO: remove unused polygons (and vertices)
    let end = SystemTime::now();
    let elapsed = end.duration_since(start);
    println!(
        "Merging took around {}s",
        elapsed.unwrap_or_default().as_secs_f32()
    );
    println!("{}", mesh_merger.to_mesh2_format());
}
