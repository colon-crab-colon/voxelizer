use std::{
    fs::File,
    io::{BufWriter, Write},
    path::Path,
};

use nalgebra::Vector3;

const MAGIC_NUMBER: &str = "VOXELSRS";

pub fn save_voxels<P: AsRef<Path>>(path: P, voxels: &[(Vector3<i32>, [u8; 4])]) {
    let mut writer = BufWriter::new(File::create(path).unwrap());

    // Write magic number
    writer.write_all(MAGIC_NUMBER.as_bytes()).unwrap();

    // Write number of voxels
    writer
        .write_all(&(voxels.len() as u64).to_le_bytes())
        .unwrap();

    // 3 Positions and 1 Color
    let mut row = [0i32; 4];

    for (v, c) in voxels {
        row[0] = v.x;
        row[1] = v.y;
        row[2] = v.z;
        row[3] = i32::from_le_bytes(*c);

        writer
            .write_all(unsafe {
                std::slice::from_raw_parts(row.as_ptr() as *const u8, size_of::<i32>() * row.len())
            })
            .unwrap();
    }
}
