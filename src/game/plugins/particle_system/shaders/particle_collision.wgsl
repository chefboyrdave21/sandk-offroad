// Particle collision compute shader
// Handles collisions between particles and scene geometry

struct Particle {
    position: vec3<f32>,
    velocity: vec3<f32>,
    color: vec4<f32>,
    size_and_rot: vec3<f32>,
    lifetime: vec2<f32>,
    random_seed: f32,
    tex_params: vec2<f32>,
    _padding: vec2<f32>,
}

struct CollisionPlane {
    normal: vec3<f32>,
    distance: f32,
    friction: f32,
    restitution: f32,
    _padding: vec2<f32>,
}

struct CollisionSphere {
    center: vec3<f32>,
    radius: f32,
    friction: f32,
    restitution: f32,
    _padding: vec2<f32>,
}

struct CollisionBox {
    center: vec3<f32>,
    half_extents: vec3<f32>,
    rotation: vec4<f32>, // quaternion
    friction: f32,
    restitution: f32>,
    _padding: vec2<f32>,
}

struct CollisionParams {
    num_planes: u32,
    num_spheres: u32,
    num_boxes: u32,
    delta_time: f32,
    collision_damping: f32,
    friction_damping: f32,
    _padding: vec2<f32>,
}

@group(0) @binding(0)
var<storage, read> particles_in: array<Particle>;

@group(0) @binding(1)
var<storage, read_write> particles_out: array<Particle>;

@group(0) @binding(2)
var<storage, read> collision_planes: array<CollisionPlane>;

@group(0) @binding(3)
var<storage, read> collision_spheres: array<CollisionSphere>;

@group(0) @binding(4)
var<storage, read> collision_boxes: array<CollisionBox>;

@group(0) @binding(5)
var<uniform> params: CollisionParams;

// Helper function to rotate a vector by a quaternion
fn quat_rotate(q: vec4<f32>, v: vec3<f32>) -> vec3<f32> {
    let qv = vec3<f32>(q.x, q.y, q.z);
    let uv = cross(qv, v);
    let uuv = cross(qv, uv);
    return v + ((uv * q.w) + uuv) * 2.0;
}

// Helper function to rotate a vector by inverse quaternion
fn quat_rotate_inv(q: vec4<f32>, v: vec3<f32>) -> vec3<f32> {
    let q_inv = vec4<f32>(-q.x, -q.y, -q.z, q.w);
    return quat_rotate(q_inv, v);
}

// Check collision with a plane and update particle velocity
fn handle_plane_collision(plane: CollisionPlane, pos: vec3<f32>, vel: vec3<f32>) -> vec3<f32> {
    let dist = dot(pos, plane.normal) - plane.distance;
    
    if (dist < 0.0) {
        // Collision response
        let normal_vel = dot(vel, plane.normal);
        if (normal_vel < 0.0) {
            // Calculate reflection vector with restitution
            let reflection = vel - (1.0 + plane.restitution) * normal_vel * plane.normal;
            
            // Apply friction
            let tangent_vel = reflection - dot(reflection, plane.normal) * plane.normal;
            return reflection - tangent_vel * plane.friction * params.friction_damping;
        }
    }
    return vel;
}

// Check collision with a sphere and update particle velocity
fn handle_sphere_collision(sphere: CollisionSphere, pos: vec3<f32>, vel: vec3<f32>) -> vec3<f32> {
    let to_center = sphere.center - pos;
    let dist = length(to_center);
    
    if (dist < sphere.radius) {
        let normal = to_center / dist;
        let normal_vel = dot(vel, normal);
        
        if (normal_vel < 0.0) {
            // Calculate reflection vector with restitution
            let reflection = vel - (1.0 + sphere.restitution) * normal_vel * normal;
            
            // Apply friction
            let tangent_vel = reflection - dot(reflection, normal) * normal;
            return reflection - tangent_vel * sphere.friction * params.friction_damping;
        }
    }
    return vel;
}

// Check collision with a box and update particle velocity
fn handle_box_collision(box: CollisionBox, pos: vec3<f32>, vel: vec3<f32>) -> vec3<f32> {
    // Transform particle position into box local space
    let local_pos = quat_rotate_inv(box.rotation, pos - box.center);
    let local_vel = quat_rotate_inv(box.rotation, vel);
    
    // Check AABB collision in local space
    let abs_pos = abs(local_pos);
    if (all(abs_pos < box.half_extents)) {
        // Find closest face
        let distances = box.half_extents - abs_pos;
        let min_dist = min(min(distances.x, distances.y), distances.z);
        
        var normal = vec3<f32>(0.0);
        if (min_dist == distances.x) {
            normal = vec3<f32>(sign(local_pos.x), 0.0, 0.0);
        } else if (min_dist == distances.y) {
            normal = vec3<f32>(0.0, sign(local_pos.y), 0.0);
        } else {
            normal = vec3<f32>(0.0, 0.0, sign(local_pos.z));
        }
        
        let normal_vel = dot(local_vel, normal);
        if (normal_vel < 0.0) {
            // Calculate reflection vector with restitution
            let reflection = local_vel - (1.0 + box.restitution) * normal_vel * normal;
            
            // Apply friction
            let tangent_vel = reflection - dot(reflection, normal) * normal;
            let final_vel = reflection - tangent_vel * box.friction * params.friction_damping;
            
            // Transform velocity back to world space
            return quat_rotate(box.rotation, final_vel);
        }
    }
    return vel;
}

@compute @workgroup_size(256)
fn update(@builtin(global_invocation_id) global_id: vec3<u32>) {
    let index = global_id.x;
    
    var particle = particles_in[index];
    var new_velocity = particle.velocity;
    
    // Check collisions with planes
    for (var i = 0u; i < params.num_planes; i = i + 1u) {
        new_velocity = handle_plane_collision(collision_planes[i], particle.position, new_velocity);
    }
    
    // Check collisions with spheres
    for (var i = 0u; i < params.num_spheres; i = i + 1u) {
        new_velocity = handle_sphere_collision(collision_spheres[i], particle.position, new_velocity);
    }
    
    // Check collisions with boxes
    for (var i = 0u; i < params.num_boxes; i = i + 1u) {
        new_velocity = handle_box_collision(collision_boxes[i], particle.position, new_velocity);
    }
    
    // Apply collision damping
    new_velocity = new_velocity * params.collision_damping;
    
    // Update particle position
    particle.position = particle.position + new_velocity * params.delta_time;
    particle.velocity = new_velocity;
    
    particles_out[index] = particle;
} 