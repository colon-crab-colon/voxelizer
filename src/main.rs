use ahash::HashMap;
use clap::Parser;
use formats::{gltf::load_gltf, ply::load_ply, voxels::save_voxels};
use indicatif::{MultiProgress, ProgressBar, ProgressStyle};
use nalgebra::Vector3;
use std::{path::PathBuf, str::FromStr, time::Instant};

pub mod bbox;
pub mod formats;
pub mod mesh;
pub mod pointcloud;

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    /// Path to mesh
    #[arg(short, long)]
    input: String,

    /// Path to output
    #[arg(short, long)]
    output: String,

    /// Resolution
    #[arg(short, long)]
    resolution: f32,

    /// x-axis rotation
    #[arg(short, long)]
    x_rotation: Option<f32>,

    /// y-axis rotation
    #[arg(short, long)]
    y_rotation: Option<f32>,

    /// z-axis rotation
    #[arg(short, long)]
    z_rotation: Option<f32>,
}

fn main() {
    let args = Args::parse();

    let input = PathBuf::from_str(&args.input).expect("Input should be a valid path");

    let output = PathBuf::from_str(&args.output).expect("Output should be a valid path");

    let rotation =
        match args.x_rotation.is_some() || args.y_rotation.is_some() || args.z_rotation.is_some() {
            false => None,
            true => Some(Vector3::new(
                args.x_rotation.unwrap_or(0f32).to_radians(),
                args.y_rotation.unwrap_or(0f32).to_radians(),
                args.z_rotation.unwrap_or(0f32).to_radians(),
            )),
        };

    let extension = input
        .extension()
        .expect("Input path doesn't have a extension")
        .to_string_lossy()
        .to_string();

    println!("Using {} resolution", args.resolution);

    match extension.as_str() {
        "gltf" | "glb" => {
            gltf(input, output, args.resolution, rotation);
        }
        "ply" => {
            ply(input, output, args.resolution, rotation);
        }
        _ => {
            eprintln!("Unrecognized extension '{}'", extension);
        }
    }
}

fn ply(input: PathBuf, output: PathBuf, resolution: f32, rotation: Option<Vector3<f32>>) {
    let start = Instant::now();
    let mut pointcloud = load_ply(&input);

    if let Some(rotation) = rotation {
        pointcloud.rotate(rotation);
    }

    println!(
        "Loaded '{}' in {:.3}s",
        input.display(),
        start.elapsed().as_secs_f32()
    );

    let bar = ProgressBar::new(0)
        .with_style(
            ProgressStyle::with_template("[{elapsed_precise}] {bar:50} {pos}/{len} {msg}").unwrap(),
        )
        .with_message("- Voxelizing...");

    let start = Instant::now();
    let voxels = pointcloud.voxelize(resolution, &bar);

    drop(bar);

    println!(
        "Voxelized point cloud in {:.3}s",
        start.elapsed().as_secs_f64()
    );

    let start = Instant::now();
    save_voxels(&output, &voxels);
    println!(
        "Saved {} voxels in file '{}' in {:.3}s!",
        voxels.len(),
        output.display(),
        start.elapsed().as_secs_f32()
    );
}

fn gltf(input: PathBuf, output: PathBuf, resolution: f32, rotation: Option<Vector3<f32>>) {
    let start = Instant::now();
    let mut meshes = load_gltf(&input);
    println!(
        "Loaded '{}' in {:.3}s",
        input.display(),
        start.elapsed().as_secs_f32()
    );

    let start = Instant::now();
    let mut voxels = Vec::new();

    let bars = MultiProgress::new();

    let scene_bar = ProgressBar::new(meshes.len() as u64)
        .with_style(
            ProgressStyle::with_template("[{elapsed_precise}] {bar:50} {pos}/{len} {msg}").unwrap(),
        )
        .with_message("- Scene");

    let scene_bar = bars.add(scene_bar);

    let mesh_bar = ProgressBar::new(0)
        .with_style(
            ProgressStyle::with_template("[{elapsed_precise}] {bar:50} {pos}/{len} {msg}").unwrap(),
        )
        .with_message("- Mesh");

    let mesh_bar = bars.add(mesh_bar);

    for mesh in meshes.iter_mut() {
        if let Some(rotation) = rotation {
            mesh.rotate(rotation);
        }

        voxels.append(&mut mesh.voxelize_shell(resolution, &mesh_bar));

        scene_bar.inc(1);
    }

    drop(mesh_bar);
    drop(scene_bar);

    println!("Voxelized scene in {:.3}s", start.elapsed().as_secs_f64());

    let start = Instant::now();
    let voxels = HashMap::from_iter(voxels);
    let voxels = voxels.into_iter().collect::<Vec<(Vector3<i32>, [u8; 4])>>();
    println!(
        "Deduplicated voxels in {:.3}s",
        start.elapsed().as_secs_f64()
    );

    let start = Instant::now();
    save_voxels(&output, &voxels);
    println!(
        "Saved {} voxels in file '{}' in {:.3}s!",
        voxels.len(),
        output.display(),
        start.elapsed().as_secs_f32()
    );
}
