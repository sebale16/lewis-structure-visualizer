use anyhow::Context;
use image::{DynamicImage, ImageBuffer, RgbaImage, RgbImage};
use wgpu::util::DeviceExt;

use crate::{model::{self, Material, Mesh, ModelVertex}, texture};

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
        let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
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
        let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("res/orbitals")
            .join(file_name);
        std::fs::read(path)?
    };

    Ok(data)
}

pub async fn load_model(
    file_name: &str,
    device: &wgpu::Device,
    queue: &wgpu::Queue,
    layout: &wgpu::BindGroupLayout,
) -> anyhow::Result<model::Model> {

    let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("res/orbitals")
        .join(file_name);

    let (document, buffers, images) = gltf::import(path.as_path())
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
            let mat_idx_opt = gltf_mat.index();

            let material_idx = if let Some(gidx) = mat_idx_opt {
                // reuse exisiting material if already created
                if let Some(existing) = materials[gidx] {
                    materials[gidx] = Some(existing);
                    existing
                } else {
                    // create new material from gltf material
                    let mat_name = gltf_mat.name().unwrap_or("material").to_string();

                    let diffuse_texture = if let Some(info) = gltf_mat.pbr_metallic_roughness().base_color_texture() {
                        let tex = info.texture();
                        let image_index = tex.source().index();
                        let img = &images[image_index];

                        let dyn_img: DynamicImage = {
                            let w = img.width as usize;
                            let h = img.height as usize;
                            let byte_len = img.pixels.len();

                            if byte_len == w * h * 4 {
                                // raw RGBA8
                                let rgba: RgbaImage = ImageBuffer::from_raw(img.width, img.height, img.pixels.clone())
                                    .context("from_raw failed for RGBA image")?;
                                DynamicImage::ImageRgba8(rgba)
                            } else if byte_len == w * h * 3 {
                                // raw RGB8 -> convert to RGBA
                                let rgb: RgbImage = ImageBuffer::from_raw(img.width, img.height, img.pixels.clone())
                                    .context("from_raw failed for RGB image")?;
                                DynamicImage::ImageRgb8(rgb)
                            } else {
                                // maybe it's encoded (png/jpg) bytes, try decode
                                image::load_from_memory(&img.pixels)
                                    .context("Failed to parse image bytes: not raw RGB(A) and not an encoded image")?
                            }
                        };

                        texture::Texture::from_image(device, queue, &dyn_img, tex.name())?
                    } else {
                        let rgba: [u8; 4] = [255, 255, 255, 255];
                        texture::Texture::from_bytes(device, queue, &rgba, "white")?
                    };

                    let bind_group = device.create_bind_group(
                        &wgpu::BindGroupDescriptor {
                            label: Some(&format!("material_bg_{}", out_materials.len())),
                            layout: layout,
                            entries: &[
                                wgpu::BindGroupEntry {
                                    binding: 0,
                                    resource: wgpu::BindingResource::TextureView(&diffuse_texture.view),
                                },
                                wgpu::BindGroupEntry {
                                    binding: 1,
                                    resource: wgpu::BindingResource::Sampler(&diffuse_texture.sampler),
                                }
                            ]
                        }
                    );

                    out_materials.push(model::Material {
                        name: mat_name,
                        diffuse_texture,
                        bind_group
                    });

                    let index = out_materials.len() - 1;
                    materials[gidx] = Some(index);
                    index
                }
            } else {
                if out_materials.is_empty() {
                    let rgba: [u8; 4] = [255, 255, 255, 255];
                    let tex = texture::Texture::from_bytes(device, queue, &rgba, "default_white")?;
                    let bind_group = device.create_bind_group(
                        &wgpu::BindGroupDescriptor {
                            label: Some("default_material_bg"),
                            layout: layout,
                            entries: &[
                                wgpu::BindGroupEntry { binding: 0, resource: wgpu::BindingResource::TextureView(&tex.view) },
                                wgpu::BindGroupEntry { binding: 1, resource: wgpu::BindingResource::Sampler(&tex.sampler) },
                            ]
                        }
                    );

                    out_materials.push(Material {
                        name: "default".to_string(),
                        diffuse_texture: tex,
                        bind_group
                    });
                }
                0
            };

            out_meshes.push(Mesh {
                name: mesh_name.clone(),
                vertex_buffer: vb,
                index_buffer: ib,
                num_elements: index_count,
                material: material_idx,
                index_format
            });
        }
    }

    Ok(model::Model { meshes: out_meshes, materials: out_materials })
}