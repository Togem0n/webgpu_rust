
#![allow(dead_code)]

// use std::f32::consts::PI;
// use cgmath::*;
use bytemuck:: {Pod, Zeroable}; 


#[repr(C)]
#[derive(Copy, Clone, Debug, Pod, Zeroable)] 
pub struct Vertex {
    position: [f32; 4], 
    color: [f32; 4],
}

fn vertex(p:[i8;3], c:[i8; 3]) -> Vertex {
    Vertex {
        position: [p[0] as f32, p[1] as f32, p[2] as f32, 1.0],
        color: [c[0] as f32, c[1] as f32, c[2] as f32, 1.0],
    }
}


pub fn create_3dline_vertices() -> [Vertex; 300]{
    let mut vertices = [
        Vertex{
            position:[0.0, 0.0, 0.0, 0.0],
            color: [255.0, 255.0, 255.0, 0.0]
        }; 
        300
    ]; 
    for i in 0..300 {
        let t = 0.1*(i as f32)/30.0;
        let x = (-t).exp()*(30.0*t).sin();
        let z = (-t).exp()*(30.0*t).cos();
        let y = 2.0*t-1.0;
            vertices[i] = Vertex{position:[x, y, z, 0.0], color: [255.0, 255.0, 255.0, 0.0] };
    }
    vertices
}

pub fn create_cube_vertices() -> Vec<Vertex> {
    let (pos, col, _uv, _normal) = cube_data();
    let mut data:Vec<Vertex> = Vec::with_capacity(pos.len());
    for i in 0..pos.len() {
        data.push(vertex(pos[i], col[i]));
    }
    data.to_vec()
}

impl Vertex {
    const ATTRIBUTES: [wgpu::VertexAttribute; 2] = wgpu::vertex_attr_array![0=>Float32x4, 1=>Float32x4]; 
    pub fn desc<'a>() -> wgpu::VertexBufferLayout<'a> {
        wgpu::VertexBufferLayout {
            array_stride: std::mem::size_of::<Vertex>() as wgpu::BufferAddress, 
            step_mode: wgpu::VertexStepMode::Vertex,
            attributes: &Self::ATTRIBUTES,
        } 
    }
}

pub fn cube_data() -> (Vec<[i8; 3]>, Vec<[i8; 3]>, Vec<[i8; 2]>, Vec<[i8; 3]>) {
    let positions = [
        // front (0, 0, 1)
        [-1, -1,  1], [1, -1,  1], [-1,  1,  1], [-1,  1,  1], [ 1, -1,  1], [ 1,  1,  1],

        // right (1, 0, 0)
        [ 1, -1,  1], [1, -1, -1], [ 1,  1,  1], [ 1,  1,  1], [ 1, -1, -1], [ 1,  1, -1],

        // back (0, 0, -1)
        [ 1, -1, -1], [-1, -1, -1], [1,  1, -1], [ 1,  1, -1], [-1, -1, -1], [-1,  1, -1],

        // left (-1, 0, 0)
        [-1, -1, -1], [-1, -1,  1], [-1,  1, -1], [-1,  1, -1], [-1, -1,  1], [-1,  1,  1],

        // top (0, 1, 0)
        [-1,  1,  1], [ 1,  1,  1], [-1,  1, -1], [-1,  1, -1], [ 1,  1,  1], [ 1,  1, -1],

        // bottom (0, -1, 0)
        [-1, -1, -1], [ 1, -1, -1], [-1, -1,  1], [-1, -1,  1], [ 1, -1, -1], [ 1, -1,  1],
    ];

    let colors = [
        // front - blue
        [0, 0, 1], [0, 0, 1], [0, 0, 1], [0, 0, 1], [0, 0, 1], [0, 0, 1],

        // right - red
        [1, 0, 0], [1, 0, 0], [1, 0, 0], [1, 0, 0], [1, 0, 0], [1, 0, 0],

        // back - yellow           
        [1, 1, 0], [1, 1, 0], [1, 1, 0], [1, 1, 0], [1, 1, 0], [1, 1, 0],

        // left - aqua
        [0, 1, 1], [0, 1, 1], [0, 1, 1], [0, 1, 1], [0, 1, 1], [0, 1, 1],

        // top - green
        [0, 1, 0], [0, 1, 0], [0, 1, 0], [0, 1, 0], [0, 1, 0], [0, 1, 0],

        // bottom - fuchsia
        [1, 0, 1], [1, 0, 1], [1, 0, 1], [1, 0, 1], [1, 0, 1], [1, 0, 1],        
    ];

    let uvs= [
        // front
        [0, 0], [1, 0], [0, 1], [0, 1], [1, 0], [1, 1],

        // right
        [0, 0], [1, 0], [0, 1], [0, 1], [1, 0], [1, 1],

        // back
        [0, 0], [1, 0], [0, 1], [0, 1], [1, 0], [1, 1],

        // left
        [0, 0], [1, 0], [0, 1], [0, 1], [1, 0], [1, 1],

        // top
        [0, 0], [1, 0], [0, 1], [0, 1], [1, 0], [1, 1],

        // bottom
        [0, 0], [1, 0], [0, 1], [0, 1], [1, 0], [1, 1],
    ];

    let normals = [
        // front 
        [0, 0, 1], [0, 0, 1], [0, 0, 1], [0, 0, 1], [0, 0, 1], [0, 0, 1],

        // right 
        [1, 0, 0], [1, 0, 0], [1, 0, 0], [1, 0, 0], [1, 0, 0], [1, 0, 0],

        // back           
        [0, 0, -1], [0, 0, -1], [0, 0, -1], [0, 0, -1], [0, 0, -1], [0, 0, -1],

        // left 
        [-1, 0, 0], [-1, 0, 0], [-1, 0, 0], [-1, 0, 0], [-1, 0, 0], [-1, 0, 0],

        // top 
        [0, 1, 0], [0, 1, 0], [0, 1, 0], [0, 1, 0], [0, 1, 0], [0, 1, 0],

        // bottom
        [0, -1, 0], [0, -1, 0], [0, -1, 0], [0, -1, 0], [0, -1, 0], [0, -1, 0],
    ];

    // return data
    (positions.to_vec(), colors.to_vec(), uvs.to_vec(), normals.to_vec())
}