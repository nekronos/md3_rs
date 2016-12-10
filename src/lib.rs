
extern crate byteorder;

use byteorder::{ReadBytesExt, LittleEndian};
use std::io::{Cursor,Read,Error,ErrorKind,Result};
use std::path::Path;
use std::fs::File;

const MD3_MAGIC: i32 = 0x33504449;
const MAX_QPATH: usize = 64;

#[derive(Debug,Copy,Clone)]
pub struct Vec3 {
    pub x: f32,
    pub y: f32,
    pub z: f32,
}

#[derive(Debug)]
pub struct Md3 {
    pub header: Md3Header,
    pub frames: Vec<Frame>,
    pub tags: Vec<Tag>,
    pub surfaces: Vec<Surface>,
}

#[derive(Debug)]
pub struct Md3Header {
    pub ident: i32,
    pub version: i32,
    pub name: String,
    pub flags: i32,
    pub num_frames: i32,
    pub num_tags: i32,
    pub num_surfaces: i32,
    pub num_skins: i32,
    pub ofs_frames: i32,
    pub ofs_tags: i32,
    pub ofs_surfaces: i32,
    pub ofs_eof: i32,
}

#[derive(Debug)]
pub struct Frame {
    pub min_bounds: Vec3,
    pub max_bounds: Vec3,
    pub local_origin: Vec3,
    pub radius: f32,
    pub name: String,
}

#[derive(Debug)]
pub struct Tag {
    pub name: String,
    pub origin: Vec3,
    pub axis: [Vec3; 3],
}

#[derive(Debug)]
pub struct SurfaceHeader {
    pub ident: i32,
    pub name: String,
    pub flags: i32,
    pub num_frames: i32,
    pub num_shaders: i32,
    pub num_verts: i32,
    pub num_triangles: i32,
    pub ofs_triangles: i32,
    pub ofs_shaders: i32,
    pub ofs_st: i32,
    pub ofs_xyznormal: i32,
    pub ofs_end: i32,
}

#[derive(Debug)]
pub struct Shader {
    pub name: String,
    pub shader_index: i32,
}

#[derive(Debug)]
pub struct Surface {
    pub header: SurfaceHeader,
    pub shaders: Vec<Shader>,
    pub triangles: Vec<Triangle>,
    pub tex_coords: Vec<TexCoord>,
    pub vertices: Vec<Vec<Vertex>>,
}

#[derive(Debug)]
pub struct Triangle {
    pub indexes: [i32; 3],
}

#[derive(Debug)]
pub struct TexCoord {
    pub st: [f32; 2],
}

#[derive(Debug)]
pub struct Vertex {
    pub x: i16,
    pub y: i16,
    pub z: i16,
    pub normal: i16,
}

impl Md3 {

    pub fn from_file<P: AsRef<Path>>(path: P) -> Result<Md3> {
        let mut bytes = Vec::new();
        let mut file = File::open(path)?;
        file.read_to_end(&mut bytes)?;
        Md3::from_bytes(&bytes)
    }

    pub fn from_bytes(bytes: &[u8]) -> Result<Md3> {
        let mut buff = Cursor::new(bytes);

        if Md3::read_s32(&mut buff) == MD3_MAGIC {

            buff.set_position(0);
            let md3_header = Md3::read_md3_header(&mut buff);

            buff.set_position(md3_header.ofs_frames as u64);
            let frames = Md3::read_many(&mut buff, Md3::read_frame, md3_header.num_frames as usize);

            buff.set_position(md3_header.ofs_tags as u64);
            let tags = Md3::read_many(&mut buff, Md3::read_tag, md3_header.num_tags as usize);

            buff.set_position(md3_header.ofs_surfaces as u64);
            let surfaces = Md3::read_many(&mut buff,
                                          Md3::read_surface,
                                          md3_header.num_surfaces as usize);

            let md3 = Md3 {
                header: md3_header,
                frames: frames,
                tags: tags,
                surfaces: surfaces,
            };

            Ok(md3)
        } else {
            Err(Error::new(ErrorKind::InvalidData, "MD3 magic not matching"))
        }
    }

    fn read_many<T, F>(buff: &mut Cursor<&[u8]>, reader: F, count: usize) -> Vec<T>
        where F: Fn(&mut Cursor<&[u8]>) -> T
    {
        let mut vec = Vec::new();
        for _ in 0..count {
            vec.push(reader(buff))
        }
        vec
    }

    fn read_s32(buff: &mut Cursor<&[u8]>) -> i32 {
        buff.read_i32::<LittleEndian>().unwrap()
    }

    fn read_s16(buff: &mut Cursor<&[u8]>) -> i16 {
        buff.read_i16::<LittleEndian>().unwrap()
    }

    fn read_f32(buff: &mut Cursor<&[u8]>) -> f32 {
        buff.read_f32::<LittleEndian>().unwrap()
    }

    fn read_vec3(buff: &mut Cursor<&[u8]>) -> Vec3 {
        Vec3 {
            x: Md3::read_f32(buff),
            y: Md3::read_f32(buff),
            z: Md3::read_f32(buff),
        }
    }

    fn read_string(buff: &mut Cursor<&[u8]>, len: usize) -> String {
        let mut bytes = Vec::new();
        for _ in 0..len {
            bytes.push(buff.read_u8().unwrap())
        }
        String::from_utf8(bytes.into_iter().take_while(|x| *x != '\0' as u8).collect()).unwrap()
    }

    fn read_md3_header(buff: &mut Cursor<&[u8]>) -> Md3Header {
        Md3Header {
            ident: Md3::read_s32(buff),
            version: Md3::read_s32(buff),
            name: Md3::read_string(buff, MAX_QPATH),
            flags: Md3::read_s32(buff),
            num_frames: Md3::read_s32(buff),
            num_tags: Md3::read_s32(buff),
            num_surfaces: Md3::read_s32(buff),
            num_skins: Md3::read_s32(buff),
            ofs_frames: Md3::read_s32(buff),
            ofs_tags: Md3::read_s32(buff),
            ofs_surfaces: Md3::read_s32(buff),
            ofs_eof: Md3::read_s32(buff),
        }
    }

    fn read_frame(buff: &mut Cursor<&[u8]>) -> Frame {
        Frame {
            min_bounds: Md3::read_vec3(buff),
            max_bounds: Md3::read_vec3(buff),
            local_origin: Md3::read_vec3(buff),
            radius: Md3::read_f32(buff),
            name: Md3::read_string(buff, 16),
        }
    }

    fn read_tag(buff: &mut Cursor<&[u8]>) -> Tag {
        Tag {
            name: Md3::read_string(buff, MAX_QPATH),
            origin: Md3::read_vec3(buff),
            axis: [Md3::read_vec3(buff), Md3::read_vec3(buff), Md3::read_vec3(buff)],
        }
    }

    fn read_surface(buff: &mut Cursor<&[u8]>) -> Surface {
        let surface_start = buff.position();
        let surface_header = Md3::read_surface_header(buff);

        buff.set_position(surface_start + surface_header.ofs_shaders as u64);
        let shaders = Md3::read_many(buff, Md3::read_shader, surface_header.num_shaders as usize);

        buff.set_position(surface_start + surface_header.ofs_triangles as u64);
        let triangles = Md3::read_many(buff,
                                       Md3::read_triangle,
                                       surface_header.num_triangles as usize);

        buff.set_position(surface_start + surface_header.ofs_st as u64);
        let tex_coords =
            Md3::read_many(buff, Md3::read_tex_coord, surface_header.num_verts as usize);

        buff.set_position(surface_start + surface_header.ofs_xyznormal as u64);
        let vertices = Md3::read_many(buff,
                                       |x| {
                                           Md3::read_many(x,
                                                          Md3::read_vertex,
                                                          surface_header.num_verts as usize)
                                       },
                                       surface_header.num_frames as usize);

        Surface {
            header: surface_header,
            shaders: shaders,
            triangles: triangles,
            tex_coords: tex_coords,
            vertices: vertices,
        }
    }

    fn read_surface_header(buff: &mut Cursor<&[u8]>) -> SurfaceHeader {
        SurfaceHeader {
            ident: Md3::read_s32(buff),
            name: Md3::read_string(buff, MAX_QPATH),
            flags: Md3::read_s32(buff),
            num_frames: Md3::read_s32(buff),
            num_shaders: Md3::read_s32(buff),
            num_verts: Md3::read_s32(buff),
            num_triangles: Md3::read_s32(buff),
            ofs_triangles: Md3::read_s32(buff),
            ofs_shaders: Md3::read_s32(buff),
            ofs_st: Md3::read_s32(buff),
            ofs_xyznormal: Md3::read_s32(buff),
            ofs_end: Md3::read_s32(buff),
        }
    }

    fn read_shader(buff: &mut Cursor<&[u8]>) -> Shader {
        Shader {
            name: Md3::read_string(buff, MAX_QPATH),
            shader_index: Md3::read_s32(buff),
        }
    }

    fn read_triangle(buff: &mut Cursor<&[u8]>) -> Triangle {
        Triangle { indexes: [Md3::read_s32(buff), Md3::read_s32(buff), Md3::read_s32(buff)] }
    }

    fn read_tex_coord(buff: &mut Cursor<&[u8]>) -> TexCoord {
        TexCoord { st: [Md3::read_f32(buff), Md3::read_f32(buff)] }
    }

    fn read_vertex(buff: &mut Cursor<&[u8]>) -> Vertex {
        Vertex {
            x: Md3::read_s16(buff),
            y: Md3::read_s16(buff),
            z: Md3::read_s16(buff),
            normal: Md3::read_s16(buff),
        }
    }
}

/*#[cfg(test)]
mod tests {

    use super::*;

    #[test]
    fn load_rocketl() {
        let bytes = include_bytes!("../rocketl.md3");
        let md3 = Md3::from_bytes(bytes).unwrap();
    }

    #[test]
    fn load_head() {
        let bytes = include_bytes!("../head.md3");
        let md3 = Md3::from_bytes(bytes).unwrap();
    }

    #[test]
    fn load_upper() {
        let bytes = include_bytes!("../upper.md3");
        let md3 = Md3::from_bytes(bytes).unwrap();
    }

    #[test]
    fn load_lower() {
        let bytes = include_bytes!("../lower.md3");
        let md3 = Md3::from_bytes(bytes).unwrap();
    }
}
*/