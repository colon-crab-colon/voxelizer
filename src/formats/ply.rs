use std::{
    fs::File,
    io::{BufRead, BufReader, Read},
    path::Path,
};

use nalgebra::Vector3;

use crate::pointcloud::PointCloud;

pub fn load_ply<P: AsRef<Path>>(path: P) -> PointCloud {
    let mut reader = BufReader::new(File::open(path.as_ref()).unwrap());

    let mut line = String::new();

    reader
        .read_line(&mut line)
        .expect("Invalid ply magic number");
    assert_eq!(line, "ply\n");
    line.clear();

    reader
        .read_line(&mut line)
        .expect("Invalid ply format, only binary_little_endian is supported");
    assert_eq!(line, "format binary_little_endian 1.0\n");
    line.clear();

    reader.read_line(&mut line).unwrap();

    let num_vertex = line
        .strip_prefix("element vertex ")
        .unwrap()
        .strip_suffix("\n")
        .unwrap()
        .parse::<usize>()
        .unwrap();

    line.clear();

    reader.read_line(&mut line).unwrap();
    assert_eq!(line, "property float x\n");
    line.clear();

    reader.read_line(&mut line).unwrap();
    assert_eq!(line, "property float y\n");
    line.clear();

    reader.read_line(&mut line).unwrap();
    assert_eq!(line, "property float z\n");
    line.clear();

    reader.read_line(&mut line).unwrap();

    assert_eq!(line, "end_header\n");

    let mut buffer: Vec<(Vector3<f32>, [u8; 4])> = Vec::with_capacity(num_vertex);

    let mut tmp = [0u8; 4];

    for _ in 0..num_vertex {
        reader.read_exact(&mut tmp).unwrap();
        let x = f32::from_le_bytes(tmp);
        reader.read_exact(&mut tmp).unwrap();
        let y = f32::from_le_bytes(tmp);
        reader.read_exact(&mut tmp).unwrap();
        let z = f32::from_le_bytes(tmp);

        if x.is_nan() || y.is_nan() || z.is_nan() {
            continue;
        }

        // Swap because the point clouds will be flipped
        buffer.push((Vector3::new(x, z, y), [255u8; 4]));
    }

    PointCloud::new(buffer)
}
