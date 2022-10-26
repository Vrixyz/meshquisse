use clap::Parser;
use meshquisse::{
    mesh_data::{merge_triangles::ConvexPolygonsMeshData, only_triangles::TriangleMeshData},
    trianglemerger::MeshMerger,
};
use parry2d::{
    math::{Point, Real},
    transformation::hertel_mehlhorn,
};
use std::{io::Read, time::SystemTime};

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// path to the mesh to merge
    #[arg(short, long)]
    path: String,
    #[arg(value_enum)]
    mode: MergeMode,
}

#[derive(clap::ValueEnum, Clone, Debug)]
enum MergeMode {
    MergeTriangles,
    Repartition,
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
    /*println!(
        "Reading took around {}s",
        elapsed.unwrap_or_default().as_secs_f32()
    );*/
    let start = SystemTime::now();
    match args.mode {
        MergeMode::MergeTriangles => {
            mesh_merger.my_merge();
            mesh_merger.remove_unused();
        }
        MergeMode::Repartition => {
            let triangle_data = TriangleMeshData::from(&ConvexPolygonsMeshData::from(&mesh_merger));
            let vertices: Vec<Point<Real>> = triangle_data
                .0
                .positions
                .iter()
                .map(|v| Point::new(v.x, v.y))
                .collect();
            // TODO: see triangulate to enumerate length / 3, take index+0,+1,+2 into a triangle
            let indices: Vec<[u32; 3]> = (0..triangle_data.0.indices.len() / 3)
                .into_iter()
                .map(|index| {
                    [
                        triangle_data.0.indices[index],
                        triangle_data.0.indices[index + 1],
                        triangle_data.0.indices[index + 2],
                    ]
                })
                .collect();

            let res: Vec<Vec<Point<Real>>> = hertel_mehlhorn(&vertices, &indices);
            println!("new polygon count after merge: {}", res.len());
            // TODO: create a mesh format 2 from that data...
        }
    }
    // TODO: remove unused polygons (and vertices)
    let end = SystemTime::now();
    let elapsed = end.duration_since(start);
    println!(
        "Merging took around {}s",
        elapsed.unwrap_or_default().as_secs_f32()
    );
    //println!("{}", mesh_merger.to_mesh2_format());
}
