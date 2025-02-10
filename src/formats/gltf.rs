use std::path::Path;

use gltf::{image::Source, mesh::Mode, Node};
use nalgebra::{Vector2, Vector3};

use crate::mesh::{texture::Texture, Mesh};

pub fn load_gltf<P: AsRef<Path>>(path: P) -> Vec<Mesh> {
    let (gltf, buffers, _) = gltf::import(&path).unwrap();

    // Extracting Nodes
    let mut nodes: Vec<Node<'_>> = Vec::new();

    for scene in gltf.scenes() {
        let mut stack: Vec<Node<'_>> = Vec::new();

        stack.extend(scene.nodes());

        while let Some(node) = stack.pop() {
            stack.extend(node.children());

            nodes.push(node);
        }
    }

    let mut meshes = Vec::new();

    // Processing Nodes
    for node in nodes {
        let mesh = match node.mesh() {
            Some(m) => m,
            None => {
                continue;
            }
        };

        for primitive in mesh.primitives() {
            if primitive.mode() != Mode::Triangles {
                panic!("Mesh contains invalid primitive mode");
            }

            let reader = primitive.reader(|buffer| Some(&buffers[buffer.index()]));

            let material = primitive.material();

            let pbr = material.pbr_metallic_roughness();

            let mut tmp = None;

            if let Some(texture) = pbr.base_color_texture() {
                let texture = texture.texture();

                let image = texture.source();

                match image.source() {
                    Source::View { view, mime_type: _ } => {
                        let buffer = &buffers[view.buffer().index()];

                        let begin = view.offset();
                        let end = begin + view.length();

                        let data = &buffer[begin..end];

                        tmp = Some(Texture::Raw(data.to_vec()))
                    }
                    Source::Uri { uri, mime_type: _ } => {
                        let mut path = path.as_ref().to_path_buf();
                        path.pop();

                        path.push(uri);

                        tmp = Some(Texture::Path(path));
                    }
                };
            }

            let vertices = reader
                .read_positions()
                .unwrap()
                .map(|v| Vector3::new(v[0], v[1], v[2]))
                .collect::<Vec<Vector3<f32>>>();

            let indices = reader
                .read_indices()
                .unwrap()
                .into_u32()
                .map(|n| n as usize)
                .collect::<Vec<usize>>();

            let mut coordinates = None;

            if tmp.is_some() {
                coordinates = Some(
                    reader
                        .read_tex_coords(0)
                        .expect("Got texture but no texture coordinates")
                        .into_f32()
                        .map(|t| Vector2::new(t[0], t[1]))
                        .collect::<Vec<Vector2<f32>>>(),
                );
            }

            meshes.push(Mesh::new(vertices, indices, coordinates, tmp));
        }
    }

    meshes
}
