struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) vert_pos: vec3<f32>,
    @location(1) index: u32
}

@vertex
fn vert(
    @builtin(vertex_index) in_vertex_index: u32,
) -> VertexOutput {
    var out: VertexOutput;
    let x = f32(1 - i32(in_vertex_index)) * 0.5;
    let y = f32(i32(in_vertex_index & 1u) * 2 - 1) * 0.5;
    out.clip_position = vec4<f32>(x, y, 0.0, 1.0);
    out.vert_pos = out.clip_position.xyz;
    out.index = in_vertex_index;
    return out;
}

const colors = array<vec4<f32>, 3>(
    vec4<f32>(1.0, 0.0, 0.0, 1.0),
    vec4<f32>(0.0, 1.0, 0.0, 1.0),
    vec4<f32>(0.0, 0.0, 1.0, 1.0),
);

@fragment
fn frag(in: VertexOutput) -> @location(0) vec4<f32> {
    switch (in.index) {
        case 0u: {return colors[0];}
        case 1u: {return colors[1];}
        case 2u: {return colors[2];}
        default: {return colors[1];}
    }
    return colors[1];
}
