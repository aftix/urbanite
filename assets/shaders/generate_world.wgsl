#import bevy_pbr::mesh_view_bindings
#import bevy_pbr::mesh_bindings

// NOTE: Bindings must come before functions that use them!
#import bevy_pbr::mesh_functions

struct Vertex {
    @location(0) position: vec3<f32>,
    @location(1) normal: vec3<f32>,
#ifdef VERTEX_UVS
    @location(2) uv: vec2<f32>,
#endif
#ifdef VERTEX_TANGENTS
    @location(3) tangent: vec4<f32>,
#endif
#ifdef VERTEX_COLORS
    @location(4) color: vec4<f32>,
#endif
#ifdef SKINNED
    @location(5) joint_indices: vec4<u32>,
    @location(6) joint_weights: vec4<f32>,
#endif
};

@group(1) @binding(11)
var elevation_map: texture_2d<f32>;
@group(1) @binding(12)
var elevation_sampler: sampler;


struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    #import bevy_pbr::mesh_vertex_output
};

@vertex
fn vertex(vertex: Vertex) -> VertexOutput {
    var out: VertexOutput;
#ifdef SKINNED
    var model = skin_model(vertex.joint_indices, vertex.joint_weights);
    out.world_normal = skin_normals(model, vertex.normal);
#else
    var model = mesh.model;
    out.world_normal = mesh_normal_local_to_world(vertex.normal);
#endif

    let r = sqrt(vertex.position.x*vertex.position.x + vertex.position.y * vertex.position.y + vertex.position.z*vertex.position.z);
    let phi = acos(vertex.position.z / r);
    let theta = atan2(vertex.position.y, vertex.position.x);

    let new_r = r + textureSampleLevel(elevation_map, elevation_sampler, vec2(phi / 6.283185, theta / 3.1415926 + 0.5), 0.0).r;
    
    let position = vec3(new_r * sin(phi) * cos(theta), new_r * sin(theta) * sin(phi), new_r * cos(phi));
    out.world_position = mesh_position_local_to_world(model, vec4<f32>(position, 1.0));
#ifdef VERTEX_UVS
    out.uv = vertex.uv;
#endif
#ifdef VERTEX_TANGENTS
    out.world_tangent = mesh_tangent_local_to_world(model, vertex.tangent);
#endif
#ifdef VERTEX_COLORS
    out.color = vertex.color;
#endif

    out.clip_position = mesh_position_world_to_clip(out.world_position);

    return out;
}

struct FragmentInput {
    @builtin(front_facing) is_front: bool,
    #import bevy_pbr::mesh_vertex_output
};

@fragment
fn fragment(in: FragmentInput) -> @location(0) vec4<f32> {
#ifdef VERTEX_COLORS
    return in.color;
#else
    return vec4<f32>(1.0, 0.0, 1.0, 1.0);
#endif
}