use std::{collections::HashMap, fs::File, io::BufReader};

use crate::app::AppData;
use anyhow::Result;
use cgmath::{vec2, vec3};

use super::vertex::Vertex;

pub fn load_model(data: &mut AppData) -> Result<()> {
    let mut reader = BufReader::new(File::open("/home/daniel/Art/viking_room.obj")?);

    // We are interested only in the Vec<Model>, not in the Vec<Material>
    let (models, _) = tobj::load_obj_buf(
        &mut reader,
        &tobj::LoadOptions {
            triangulate: true,
            ..Default::default()
        },
        |_| Ok(Default::default()),
    )?;

    let mut unique_vertices = HashMap::new();

    for model in models {
        for index in &model.mesh.indices {
            // Positions are stored as a flat array in the obj format:
            // [x1, y1, z1, x2, y2, z2, x3, y3, z3, ...]
            let pos_offset = (3 * index) as usize;

            // Texture coordinates are stored as a flat array as well:
            // [u1, v1, u2, v2, u3, v3, ...]
            let tex_coord_offset = (2 * index) as usize;

            let vertex = Vertex {
                pos: vec3(
                    model.mesh.positions[pos_offset],
                    model.mesh.positions[pos_offset + 1],
                    model.mesh.positions[pos_offset + 2],
                ),
                color: vec3(1.0, 1.0, 1.0),
                tex_coord: vec2(
                    model.mesh.texcoords[tex_coord_offset],
                    // The OBJ format assumes a coordinate system where a vertical coordinate of 0 means the bottom
                    // of the image, but we've uploaded our image into Vulkan in a top to bottom orientation where 0
                    // means the top of the image. This can be solved by flipping the vertical component of the texture.
                    1.0 - model.mesh.texcoords[tex_coord_offset + 1],
                ),
            };

            if let Some(index) = unique_vertices.get(&vertex) {
                data.indices.push(*index as u32);
            } else {
                let index = data.vertices.len();
                data.vertices.push(vertex);
                data.indices.push(index as u32);
                unique_vertices.insert(vertex, index);
            }
        }
    }

    Ok(())
}
