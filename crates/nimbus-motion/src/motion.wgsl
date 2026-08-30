struct NimbusMotionUniform {
    previous_camera_pos: vec3<f32>,
    history_blend: f32,
    previous_camera_forward: vec3<f32>,
    history_valid: u32,
    previous_camera_right: vec3<f32>,
    frame: u32,
    previous_camera_up: vec3<f32>,
    previous_elapsed_seconds: f32,
}

fn nimbus_reproject_history(current_world: vec3<f32>, current_uv: vec2<f32>) -> vec3<f32> {
    let elapsed = max(clouds.elapsed_seconds - temporal.previous_elapsed_seconds, 0.0);
    let previous_world = current_world
        + vec3<f32>(clouds.movement.x, 0.0, clouds.movement.y) * elapsed;
    let delta = previous_world - temporal.previous_camera_pos;
    let forward_distance = dot(delta, temporal.previous_camera_forward);
    if (forward_distance <= 0.001) {
        return vec3<f32>(-1.0, -1.0, 1.0);
    }
    let right_scale = max(length(temporal.previous_camera_right), 0.0001);
    let up_scale = max(length(temporal.previous_camera_up), 0.0001);
    let screen_x = dot(delta, normalize(temporal.previous_camera_right))
        / (forward_distance * right_scale);
    let screen_y = dot(delta, normalize(temporal.previous_camera_up))
        / (forward_distance * up_scale);
    let uv = vec2<f32>(0.5 + 0.5 * screen_x, 0.5 - 0.5 * screen_y);
    return vec3<f32>(uv, length(uv - current_uv));
}
