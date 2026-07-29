//! Frostbite 2 MeshSet RES → OBJ/SMD (IceBloc-compatible layout).

use std::io::Cursor;

const MAX_LODS: usize = 5;

#[derive(Debug, Clone)]
pub struct MeshVertex {
    pub position: [f32; 3],
    pub normal: [f32; 3],
    pub uv: [f32; 2],
    pub bone_indices: [i32; 4],
    pub bone_weights: [f32; 4],
}

#[derive(Debug, Clone)]
pub struct MeshSurface {
    pub name: String,
    pub vertices: Vec<MeshVertex>,
    pub faces: Vec<[u32; 3]>,
    pub skinned: bool,
}

#[derive(Debug, Clone)]
pub struct DecodedMeshSet {
    pub name: String,
    pub surfaces: Vec<MeshSurface>,
}

impl DecodedMeshSet {
    pub fn total_vertices(&self) -> usize {
        self.surfaces.iter().map(|s| s.vertices.len()).sum()
    }

    pub fn total_faces(&self) -> usize {
        self.surfaces.iter().map(|s| s.faces.len()).sum()
    }

    pub fn positions(&self) -> Vec<[f32; 3]> {
        let mut out = Vec::new();
        for s in &self.surfaces {
            for v in &s.vertices {
                out.push(v.position);
            }
        }
        out
    }

    pub fn to_obj(&self) -> String {
        let mut out = String::new();
        out.push_str(&format!("# FrostEx MeshSet: {}\n", self.name));
        let mut vert_base = 1u32;
        for (si, surf) in self.surfaces.iter().enumerate() {
            let mat = if surf.name.is_empty() {
                format!("subset_{si}")
            } else {
                sanitize_obj_name(&surf.name)
            };
            out.push_str(&format!("o {mat}\n"));
            out.push_str(&format!("g {mat}\n"));
            out.push_str(&format!("usemtl {mat}\n"));
            for v in &surf.vertices {
                out.push_str(&format!(
                    "v {} {} {}\n",
                    v.position[0], v.position[1], v.position[2]
                ));
            }
            for v in &surf.vertices {
                out.push_str(&format!("vt {} {}\n", v.uv[0], v.uv[1]));
            }
            for v in &surf.vertices {
                out.push_str(&format!(
                    "vn {} {} {}\n",
                    v.normal[0], v.normal[1], v.normal[2]
                ));
            }
            for f in &surf.faces {
                let a = vert_base + f[0];
                let b = vert_base + f[1];
                let c = vert_base + f[2];
                out.push_str(&format!("f {a}/{a}/{a} {b}/{b}/{b} {c}/{c}/{c}\n"));
            }
            vert_base += surf.vertices.len() as u32;
        }
        out
    }

    pub fn to_smd(&self) -> String {
        let mut out = String::new();
        out.push_str("version 1\n");
        out.push_str("nodes\n");
        out.push_str("0 \"root\" -1\n");
        out.push_str("end\n");
        out.push_str("skeleton\n");
        out.push_str("time 0\n");
        out.push_str("0 0 0 0 0 0 0\n");
        out.push_str("end\n");
        out.push_str("triangles\n");
        for (si, surf) in self.surfaces.iter().enumerate() {
            let mat = if surf.name.is_empty() {
                format!("subset_{si}")
            } else {
                sanitize_obj_name(&surf.name)
            };
            for f in &surf.faces {
                out.push_str(&mat);
                out.push('\n');
                for &idx in f {
                    let v = &surf.vertices[idx as usize];
                    let (bi, bw) = if surf.skinned {
                        (v.bone_indices[0].max(0), v.bone_weights[0])
                    } else {
                        (0, 1.0)
                    };
                    out.push_str(&format!(
                        "0 {} {} {} {} {} {} {} {} {} {} {}\n",
                        v.position[0],
                        v.position[1],
                        v.position[2],
                        v.normal[0],
                        v.normal[1],
                        v.normal[2],
                        v.uv[0],
                        v.uv[1],
                        bi,
                        bw,
                        0
                    ));
                }
            }
        }
        out.push_str("end\n");
        out
    }
}

/// Decode a MeshSet RES, loading streaming LOD chunks via `get_chunk`.
pub fn decode_meshset(
    res: &[u8],
    get_chunk: &dyn Fn(&[u8; 16]) -> Result<Vec<u8>, String>,
) -> Result<DecodedMeshSet, String> {
    if res.len() < 0x60 {
        return Err("MeshSet RES too short".into());
    }
    let mut c = Cursor::new(res);
    let mesh_type = read_u32(&mut c)?;
    let _flags = read_u32(&mut c)?;
    let lod_count = read_i32(&mut c)?;
    let _total_subset = read_i32(&mut c)?;
    let _bb_min = read_vec4(&mut c)?;
    let _bb_max = read_vec4(&mut c)?;
    if !(1..=MAX_LODS as i32).contains(&lod_count) {
        return Err(format!("invalid MeshSet lodCount={lod_count} (type={mesh_type})"));
    }

    let mut lod_ptrs = [0i32; MAX_LODS];
    for i in 0..MAX_LODS {
        lod_ptrs[i] = read_i32(&mut c)?;
        let _pad = read_i32(&mut c)?;
    }
    let name_ptr = read_i32(&mut c)?;
    let _name_pad = read_i32(&mut c)?;
    let _short_ptr = read_i32(&mut c)?;
    let _short_pad = read_i32(&mut c)?;
    let _name_hash = read_i32(&mut c)?;
    let _pad = read_i32(&mut c)?;

    let name = read_reloc_string(res, name_ptr).unwrap_or_else(|| "mesh".into());

    let mut surfaces = Vec::new();
    // Export LOD0 only (IceBloc exports all LODs as separate meshes; we merge LOD0 subsets).
    let lod0 = lod_ptrs[0];
    if lod0 <= 0 || lod0 as usize >= res.len() {
        return Err("MeshSet LOD0 pointer invalid".into());
    }

    let layout = read_mesh_layout(res, lod0 as usize)?;
    let chunk_data = if layout.data_chunk_id.iter().any(|b| *b != 0) {
        get_chunk(&layout.data_chunk_id).unwrap_or_default()
    } else {
        Vec::new()
    };
    let geom: &[u8] = if !chunk_data.is_empty() {
        &chunk_data
    } else if layout.embedded_ptr > 0 && (layout.embedded_ptr as usize) < res.len() {
        &res[layout.embedded_ptr as usize..]
    } else if layout.aux_vertex_index_data_offset > 0
        && (layout.aux_vertex_index_data_offset as usize) < res.len()
    {
        &res[layout.aux_vertex_index_data_offset as usize..]
    } else {
        return Err(format!(
            "MeshSet '{}': no streaming chunk {} and no embedded vertex data",
            name,
            hex::encode(layout.data_chunk_id)
        ));
    };

    if layout.subsets_ptr <= 0 {
        return Err("MeshSet has no subsets".into());
    }
    let mut subset_off = layout.subsets_ptr as usize;
    for si in 0..layout.sub_count.max(0) as usize {
        let subset = read_mesh_subset(res, subset_off)?;
        subset_off += 0x94; // MeshSubset size
        if subset.vertex_count <= 0 || subset.primitive_count <= 0 {
            continue;
        }
        let surface = decode_subset(&subset, geom, layout.vertex_data_size, layout.index_format_16)?;
        if !surface.vertices.is_empty() {
            let _ = si;
            surfaces.push(surface);
        }
    }

    if surfaces.is_empty() {
        return Err(format!("MeshSet '{name}': parsed 0 surfaces"));
    }
    Ok(DecodedMeshSet { name, surfaces })
}

struct MeshLayoutInfo {
    sub_count: i32,
    subsets_ptr: i32,
    index_format_16: bool,
    vertex_data_size: i32,
    data_chunk_id: [u8; 16],
    aux_vertex_index_data_offset: i32,
    embedded_ptr: i32,
}

struct MeshSubsetInfo {
    material_name: String,
    primitive_count: i32,
    start_index: i32,
    vertex_offset: i32,
    vertex_count: i32,
    vertex_stride: u8,
    bone_count: u8,
    bone_indices: Vec<i16>,
    elements: Vec<GeoElement>,
}

#[derive(Clone, Copy)]
struct GeoElement {
    usage: u8,
    format: u8,
    offset: u8,
}

fn read_mesh_layout(data: &[u8], at: usize) -> Result<MeshLayoutInfo, String> {
    let mut c = Cursor::new(&data[at..]);
    let _ty = read_u32(&mut c)?;
    let sub_count = read_i32(&mut c)?;
    let subsets_ptr = read_i32(&mut c)?;
    let _subsets_pad = read_i32(&mut c)?;
    // 4× RelocArray (size + reloc ptr) = 4 × (4+8) = 48
    for _ in 0..4 {
        let _size = read_u32(&mut c)?;
        let _ptr = read_i32(&mut c)?;
        let _pad = read_i32(&mut c)?;
    }
    let _flags = read_u32(&mut c)?;
    let index_fmt = read_i32(&mut c)?;
    let _index_data_size = read_i32(&mut c)?;
    let vertex_data_size = read_i32(&mut c)?;
    let _edge_data_size = read_i32(&mut c)?;
    let mut data_chunk_id = [0u8; 16];
    let pos = c.position() as usize;
    if pos + 16 > data[at..].len() {
        return Err("MeshLayout truncated at chunk id".into());
    }
    data_chunk_id.copy_from_slice(&data[at + pos..at + pos + 16]);
    c.set_position((pos + 16) as u64);
    let aux_vertex_index_data_offset = read_i32(&mut c)?;
    let embedded_ptr = read_i32(&mut c)?;
    let _embedded_pad = read_i32(&mut c)?;
    Ok(MeshLayoutInfo {
        sub_count,
        subsets_ptr,
        index_format_16: index_fmt == 0,
        vertex_data_size,
        data_chunk_id,
        aux_vertex_index_data_offset,
        embedded_ptr,
    })
}

fn read_mesh_subset(data: &[u8], at: usize) -> Result<MeshSubsetInfo, String> {
    if at + 0x94 > data.len() {
        return Err("MeshSubset OOB".into());
    }
    let mut c = Cursor::new(&data[at..]);
    let _geo_decl_ptr = read_i32(&mut c)?;
    let _geo_decl_pad = read_i32(&mut c)?;
    let mat_ptr = read_i32(&mut c)?;
    let _mat_pad = read_i32(&mut c)?;
    let _material_index = read_i32(&mut c)?;
    let primitive_count = read_i32(&mut c)?;
    let start_index = read_i32(&mut c)?;
    let vertex_offset = read_i32(&mut c)?;
    let vertex_count = read_i32(&mut c)?;
    let vertex_stride = read_u8(&mut c)?;
    let _prim_type = read_u8(&mut c)?;
    let _bones_per_vertex = read_u8(&mut c)?;
    let bone_count = read_u8(&mut c)?;
    let bone_ptr = read_i32(&mut c)?;
    let _bone_pad = read_i32(&mut c)?;

    let mut elements = Vec::new();
    for _ in 0..16 {
        let usage = read_u8(&mut c)?;
        let format = read_u8(&mut c)?;
        let offset = read_u8(&mut c)?;
        let _stream = read_u8(&mut c)?;
        if usage != 0 || format != 0 {
            elements.push(GeoElement {
                usage,
                format,
                offset,
            });
        }
    }
    // streams (4×2) + counts + pad
    for _ in 0..4 {
        let _stride = read_u8(&mut c)?;
        let _class = read_u8(&mut c)?;
    }
    let element_count = read_u8(&mut c)? as usize;
    let _stream_count = read_u8(&mut c)?;
    let _p0 = read_u8(&mut c)?;
    let _p1 = read_u8(&mut c)?;
    if element_count > 0 && element_count <= elements.len() {
        elements.truncate(element_count);
    }
    for _ in 0..6 {
        let _ = read_f32(&mut c)?;
    }

    let material_name = read_reloc_string(data, mat_ptr).unwrap_or_default();
    let mut bone_indices = Vec::new();
    if bone_count > 0 && bone_ptr > 0 {
        let bp = bone_ptr as usize;
        for i in 0..bone_count as usize {
            let off = bp + i * 2;
            if off + 2 <= data.len() {
                bone_indices.push(i16::from_le_bytes([data[off], data[off + 1]]));
            }
        }
    }

    Ok(MeshSubsetInfo {
        material_name,
        primitive_count,
        start_index,
        vertex_offset,
        vertex_count,
        vertex_stride,
        bone_count,
        bone_indices,
        elements,
    })
}

fn decode_subset(
    subset: &MeshSubsetInfo,
    geom: &[u8],
    vertex_data_size: i32,
    index16: bool,
) -> Result<MeshSurface, String> {
    let stride = subset.vertex_stride as usize;
    if stride == 0 {
        return Err("zero vertex stride".into());
    }
    let pos_el = find_usage(&subset.elements, 0x01); // Pos
    let nor_el = find_usage(&subset.elements, 0x06); // Normal
    let uv_el = find_usage(&subset.elements, 0x21); // TexCoord0
    let bi_el = find_usage(&subset.elements, 0x02); // BoneIndices
    let bw_el = find_usage(&subset.elements, 0x04); // BoneWeights
    let skinned = subset.bone_count > 0;

    let mut vertices = Vec::with_capacity(subset.vertex_count as usize);
    for vi in 0..subset.vertex_count as usize {
        let base = subset.vertex_offset as usize + vi * stride;
        if base + stride > geom.len() {
            break;
        }
        let vtx = &geom[base..base + stride];
        let position = read_element(vtx, pos_el).unwrap_or([0.0, 0.0, 0.0, 0.0]);
        let normal = read_element(vtx, nor_el).unwrap_or([0.0, 1.0, 0.0, 0.0]);
        let uv = read_element(vtx, uv_el).unwrap_or([0.0, 0.0, 0.0, 0.0]);
        let mut bone_indices = [0i32; 4];
        let mut bone_weights = [0.0f32, 0.0, 0.0, 1.0];
        if skinned {
            let bi = read_element(vtx, bi_el).unwrap_or([-1.0, -1.0, -1.0, -1.0]);
            let bw = read_element(vtx, bw_el).unwrap_or([1.0, 0.0, 0.0, 0.0]);
            for i in 0..4 {
                let local = bi[i] as i32;
                bone_indices[i] = if local >= 0 && (local as usize) < subset.bone_indices.len() {
                    subset.bone_indices[local as usize] as i32
                } else {
                    0
                };
                bone_weights[i] = bw[i];
            }
        }
        vertices.push(MeshVertex {
            position: [position[0], position[1], position[2]],
            normal: [normal[0], normal[1], normal[2]],
            uv: [uv[0], 1.0 - uv[1]],
            bone_indices,
            bone_weights,
        });
    }

    let index_start = vertex_data_size as usize + subset.start_index as usize * if index16 { 2 } else { 4 };
    let mut faces = Vec::with_capacity(subset.primitive_count as usize);
    for pi in 0..subset.primitive_count as usize {
        if index16 {
            let off = index_start + pi * 6;
            if off + 6 > geom.len() {
                break;
            }
            let a = u16::from_le_bytes([geom[off], geom[off + 1]]) as u32;
            let b = u16::from_le_bytes([geom[off + 2], geom[off + 3]]) as u32;
            let c = u16::from_le_bytes([geom[off + 4], geom[off + 5]]) as u32;
            if (a as usize) < vertices.len()
                && (b as usize) < vertices.len()
                && (c as usize) < vertices.len()
            {
                faces.push([a, b, c]);
            }
        } else {
            let off = index_start + pi * 12;
            if off + 12 > geom.len() {
                break;
            }
            let a = u32::from_le_bytes(geom[off..off + 4].try_into().unwrap());
            let b = u32::from_le_bytes(geom[off + 4..off + 8].try_into().unwrap());
            let c = u32::from_le_bytes(geom[off + 8..off + 12].try_into().unwrap());
            if (a as usize) < vertices.len()
                && (b as usize) < vertices.len()
                && (c as usize) < vertices.len()
            {
                faces.push([a, b, c]);
            }
        }
    }

    Ok(MeshSurface {
        name: subset.material_name.clone(),
        vertices,
        faces,
        skinned,
    })
}

fn find_usage(elements: &[GeoElement], usage: u8) -> Option<GeoElement> {
    elements.iter().copied().find(|e| e.usage == usage)
}

fn read_element(vtx: &[u8], el: Option<GeoElement>) -> Option<[f32; 4]> {
    let el = el?;
    let off = el.offset as usize;
    if off >= vtx.len() {
        return None;
    }
    let d = &vtx[off..];
    Some(match el.format {
        0x01 => [read_f32_at(d, 0)?, 0.0, 0.0, 0.0], // Float
        0x02 => [read_f32_at(d, 0)?, read_f32_at(d, 4)?, 0.0, 0.0], // Float2
        0x03 => [
            read_f32_at(d, 0)?,
            read_f32_at(d, 4)?,
            read_f32_at(d, 8)?,
            0.0,
        ], // Float3
        0x04 => [
            read_f32_at(d, 0)?,
            read_f32_at(d, 4)?,
            read_f32_at(d, 8)?,
            read_f32_at(d, 12)?,
        ], // Float4
        0x05 => [half_to_f32(read_u16_at(d, 0)?)?, 0.0, 0.0, 0.0],
        0x06 => [
            half_to_f32(read_u16_at(d, 0)?)?,
            half_to_f32(read_u16_at(d, 2)?)?,
            0.0,
            0.0,
        ],
        0x07 => [
            half_to_f32(read_u16_at(d, 0)?)?,
            half_to_f32(read_u16_at(d, 2)?)?,
            half_to_f32(read_u16_at(d, 4)?)?,
            0.0,
        ],
        0x08 => [
            half_to_f32(read_u16_at(d, 0)?)?,
            half_to_f32(read_u16_at(d, 2)?)?,
            half_to_f32(read_u16_at(d, 4)?)?,
            half_to_f32(read_u16_at(d, 6)?)?,
        ],
        0x0A => [
            d.first().copied().unwrap_or(0) as i8 as f32,
            d.get(1).copied().unwrap_or(0) as i8 as f32,
            d.get(2).copied().unwrap_or(0) as i8 as f32,
            d.get(3).copied().unwrap_or(0) as i8 as f32,
        ], // Byte4
        0x0B => [
            d.first().copied().unwrap_or(0) as i8 as f32 / 127.0,
            d.get(1).copied().unwrap_or(0) as i8 as f32 / 127.0,
            d.get(2).copied().unwrap_or(0) as i8 as f32 / 127.0,
            d.get(3).copied().unwrap_or(0) as i8 as f32 / 127.0,
        ],
        0x0C => [
            d.first().copied().unwrap_or(0) as f32,
            d.get(1).copied().unwrap_or(0) as f32,
            d.get(2).copied().unwrap_or(0) as f32,
            d.get(3).copied().unwrap_or(0) as f32,
        ], // UByte4
        0x0D => [
            d.first().copied().unwrap_or(0) as f32 / 255.0,
            d.get(1).copied().unwrap_or(0) as f32 / 255.0,
            d.get(2).copied().unwrap_or(0) as f32 / 255.0,
            d.get(3).copied().unwrap_or(0) as f32 / 255.0,
        ],
        0x0F => [
            read_i16_at(d, 0)? as f32,
            read_i16_at(d, 2)? as f32,
            0.0,
            0.0,
        ], // Short2
        0x11 => [
            read_i16_at(d, 0)? as f32,
            read_i16_at(d, 2)? as f32,
            read_i16_at(d, 4)? as f32,
            read_i16_at(d, 6)? as f32,
        ],
        0x13 => [
            read_i16_at(d, 0)? as f32 / 32767.0,
            read_i16_at(d, 2)? as f32 / 32767.0,
            0.0,
            0.0,
        ],
        0x15 => [
            read_i16_at(d, 0)? as f32 / 32767.0,
            read_i16_at(d, 2)? as f32 / 32767.0,
            read_i16_at(d, 4)? as f32 / 32767.0,
            read_i16_at(d, 6)? as f32 / 32767.0,
        ],
        0x16 => [
            read_u16_at(d, 0)? as f32,
            read_u16_at(d, 2)? as f32,
            0.0,
            0.0,
        ],
        0x17 => [
            read_u16_at(d, 0)? as f32,
            read_u16_at(d, 2)? as f32,
            read_u16_at(d, 4)? as f32,
            read_u16_at(d, 6)? as f32,
        ],
        0x18 => [
            read_u16_at(d, 0)? as f32 / 65535.0,
            read_u16_at(d, 2)? as f32 / 65535.0,
            0.0,
            0.0,
        ],
        0x19 => [
            read_u16_at(d, 0)? as f32 / 65535.0,
            read_u16_at(d, 2)? as f32 / 65535.0,
            read_u16_at(d, 4)? as f32 / 65535.0,
            read_u16_at(d, 6)? as f32 / 65535.0,
        ],
        _ => return None,
    })
}

fn read_reloc_string(data: &[u8], ptr: i32) -> Option<String> {
    if ptr <= 0 {
        return None;
    }
    let start = ptr as usize;
    if start >= data.len() {
        return None;
    }
    let end = data[start..]
        .iter()
        .position(|&b| b == 0)
        .map(|i| start + i)
        .unwrap_or(data.len());
    let s = String::from_utf8_lossy(&data[start..end]).into_owned();
    if s.is_empty() {
        None
    } else {
        Some(s)
    }
}

fn sanitize_obj_name(name: &str) -> String {
    name.chars()
        .map(|c| {
            if c.is_ascii_alphanumeric() || c == '_' || c == '-' || c == '.' {
                c
            } else {
                '_'
            }
        })
        .collect()
}

fn half_to_f32(h: u16) -> Option<f32> {
    // IEEE 754 half → f32
    let sign = ((h >> 15) & 1) as u32;
    let exp = ((h >> 10) & 0x1F) as u32;
    let mant = (h & 0x3FF) as u32;
    let bits = if exp == 0 {
        if mant == 0 {
            sign << 31
        } else {
            // subnormal
            let mut m = mant;
            let mut e = 127 - 15 + 1;
            while m & 0x400 == 0 {
                m <<= 1;
                e -= 1;
            }
            m &= 0x3FF;
            (sign << 31) | ((e as u32) << 23) | (m << 13)
        }
    } else if exp == 31 {
        (sign << 31) | (0xFF << 23) | (mant << 13)
    } else {
        (sign << 31) | ((exp + 127 - 15) << 23) | (mant << 13)
    };
    Some(f32::from_bits(bits))
}

fn read_u8(c: &mut Cursor<&[u8]>) -> Result<u8, String> {
    let pos = c.position() as usize;
    let data = *c.get_ref();
    let v = *data.get(pos).ok_or("u8 EOF")?;
    c.set_position((pos + 1) as u64);
    Ok(v)
}

fn read_i32(c: &mut Cursor<&[u8]>) -> Result<i32, String> {
    Ok(read_u32(c)? as i32)
}

fn read_u32(c: &mut Cursor<&[u8]>) -> Result<u32, String> {
    let pos = c.position() as usize;
    let data = *c.get_ref();
    if pos + 4 > data.len() {
        return Err("u32 EOF".into());
    }
    let v = u32::from_le_bytes(data[pos..pos + 4].try_into().unwrap());
    c.set_position((pos + 4) as u64);
    Ok(v)
}

fn read_f32(c: &mut Cursor<&[u8]>) -> Result<f32, String> {
    Ok(f32::from_bits(read_u32(c)?))
}

fn read_vec4(c: &mut Cursor<&[u8]>) -> Result<[f32; 4], String> {
    Ok([read_f32(c)?, read_f32(c)?, read_f32(c)?, read_f32(c)?])
}

fn read_f32_at(d: &[u8], off: usize) -> Option<f32> {
    if off + 4 > d.len() {
        return None;
    }
    Some(f32::from_bits(u32::from_le_bytes(
        d[off..off + 4].try_into().ok()?,
    )))
}

fn read_u16_at(d: &[u8], off: usize) -> Option<u16> {
    if off + 2 > d.len() {
        return None;
    }
    Some(u16::from_le_bytes(d[off..off + 2].try_into().ok()?))
}

fn read_i16_at(d: &[u8], off: usize) -> Option<i16> {
    Some(read_u16_at(d, off)? as i16)
}

/// True when RES looks like a MeshSet header (for rip routing).
pub fn looks_like_meshset(data: &[u8]) -> bool {
    if data.len() < 0x60 {
        return false;
    }
    let mesh_type = u32::from_le_bytes(data[0..4].try_into().unwrap());
    let lod_count = i32::from_le_bytes(data[8..12].try_into().unwrap());
    (mesh_type <= 2) && (1..=5).contains(&lod_count)
}
