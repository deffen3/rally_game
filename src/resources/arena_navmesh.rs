use navmesh::{NavMesh};

//These structures are just for my game, so that I can work with the data in f32 and usize types
pub struct ArenaNavMesh {
    pub vertices: Vec<(f32, f32, f32)>, //(x,y,z)
    pub triangles: Vec<(usize, usize, usize)>, //(vertex 1, vertex_2, vertex_3)
}

pub struct ArenaInvertedNavMesh {
    pub vertices: Vec<(f32, f32, f32)>, //(x,y,z)
    pub triangles: Vec<(usize, usize, usize)>, //(vertex 1, vertex_2, vertex_3)
}

//This is the one actually used by the navmesh library, using special nav mesh library types
pub struct ArenaNavMeshFinal {
    pub mesh: Option<NavMesh>,
}