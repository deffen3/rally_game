pub struct ArenaNavMesh {
    pub vertices: Vec<(f32, f32, f32)>, //(x,y,z)
    pub triangles: Vec<(usize, usize, usize)>, //(vertex 1, vertex_2, vertex_3)
}