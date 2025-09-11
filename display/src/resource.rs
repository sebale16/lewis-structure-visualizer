use std::io::{BufReader, Cursor};

use anyhow::Context;
use wgpu::util::DeviceExt;

use crate::{model::{self, ModelVertex, Vertex}, texture};

#[cfg(target_arch = "wasm32")]
fn format_url(file_name: &str) -> reqwest::Url {
    let window = web_sys::window().unwrap();
    let location = window.location();
    let mut origin = location.origin().unwrap();
    if !origin.ends_with("display") {
        origin = format!("{}/display", origin);
    }
    let base = reqwest::Url::parse(&format!("{}/", origin,)).unwrap();
    base.join(file_name).unwrap()
}

pub async fn load_string(file_name: &str) -> anyhow::Result<String> {
    #[cfg(target_arch = "wasm32")]
    let txt = {
        let url = format_url(file_name);
        reqwest::get(url).await?.text().await?
    };
    #[cfg(not(target_arch = "wasm32"))]
    let txt = {
        let path = std::path::Path::new(env!("OUT_DIR"))
            .join("res/orbitals")
            .join(file_name);
        std::fs::read_to_string(path)?
    };

    Ok(txt)
}

pub async fn load_binary(file_name: &str) -> anyhow::Result<Vec<u8>> {
    #[cfg(target_arch = "wasm32")]
    let data = {
        let url = format_url(file_name);
        reqwest::get(url).await?.bytes().await?.to_vec()
    };
    #[cfg(not(target_arch = "wasm32"))]
    let data = {
        let path = std::path::Path::new(env!("OUT_DIR"))
            .join("res/orbitals")
            .join(file_name);
        std::fs::read(path)?
    };

    Ok(data)
}

pub async fn load_texture(
    file_name: &str,
    device: &wgpu::Device,
    queue: &wgpu::Queue,
) -> anyhow::Result<texture::Texture> {
    let data = load_binary(file_name).await?;
    texture::Texture::from_bytes(device, queue, &data, file_name)
}

pub async fn load_model(
    file_name: &str,
    device: &wgpu::Device,
    queue: &wgpu::Queue,
    layout: &wgpu::BindGroupLayout,
) -> anyhow::Result<model::Model> {
    let path = std::path::Path::new(env!("OUT_DIR"))
        .join("res/orbitals")
        .join(file_name);
    let (document, buffers, images) = gltf::import(path)
        .with_context(|| format!("Failed to import gltf: {}", path.display()))?;

    let mut materials: Vec<Option<usize>> = vec![None; document.materials().len()];
    let mut out_materials: Vec<model::Material> = Vec::new();
    let mut out_meshes: Vec<model::Mesh> = Vec::new();

    for mesh in document.meshes() {
        let mesh_name = mesh.name().unwrap_or("gltf_mesh").to_string();
        for primitive in mesh.primitives() {
            let reader = primitive.reader(|buffer| Some(&buffers[buffer.index()].0));

            let positions: Vec<[f32; 3]> = reader.read_positions()
                .context("Primitive missing positions")?
                .collect();
    
            let normals: Vec<[f32; 3]> = match reader.read_normals() {
                Some(iter)  => iter.collect(),
                None => vec![[0.0, 0.0, 0.0]; positions.len()],
            };

            let texcoords: Vec<[f32; 2]> = match reader.read_tex_coords(0).map(|t| t.into_f32()) {
                Some(iter) => iter.collect(),
                None => vec![[0.0, 0.0]; positions.len()],
            };

            let mut vertices: Vec<ModelVertex> = Vec::with_capacity(positions.len());
            for i in 0..positions.len() {
                vertices.push(ModelVertex {
                    position: positions[i],
                    tex_coords: texcoords[i],
                    normal: normals[i]
                });
            }

            let (index_buffer_bytes, index_format, index_count) = {
                if let Some(indices) = reader.read_indices() {
                    let indices_u32: Vec<u32> = indices.into_u32().collect();
                    let count = indices_u32.len() as u32;

                    // convert all indices to u16 if possible to save memory
                    if indices_u32.iter().all(|&i| i <= u16::MAX as u32) {
                        let u16s: Vec<u16> = indices_u32.iter().map(|&i| i as u16).collect();
                        (bytemuck::cast_slice(&u16s).to_vec(), wgpu::IndexFormat::Uint16, count)
                    } else {
                        (bytemuck::cast_slice(&indices_u32).to_vec(), wgpu::IndexFormat::Uint32, count)
                    }
                } else {
                    let count = vertices.len() as u32;
                    let u32s: Vec<u32> = (0..count).collect();
                    (bytemuck::cast_slice(&u32s).to_vec(), wgpu::IndexFormat::Uint32, count)
                }
            };

            let vb = device.create_buffer_init(
                &wgpu::util::BufferInitDescriptor {
                    label: Some(&format!("{}-vertex", mesh_name)),
                    contents: bytemuck::cast_slice(&vertices),
                    usage: wgpu::BufferUsages::VERTEX
                }
            );

            let ib = device.create_buffer_init(
                &wgpu::util::BufferInitDescriptor {
                    label: Some(&format!("{}-index", mesh_name)),
                    contents: &index_buffer_bytes,
                    usage: wgpu::BufferUsages::INDEX
                }
            );

            let gltf_mat = primitive.material();
            let max_idx_opt = gltf_mat.index();

            let 

        }
    }

    Ok(model::Model { meshes: (), materials: () })
}