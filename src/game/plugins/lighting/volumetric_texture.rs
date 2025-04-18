use bevy::{
    prelude::*,
    render::{
        render_resource::{
            Extent3d, TextureDescriptor, TextureDimension, TextureFormat, TextureUsages,
        },
        renderer::RenderDevice,
        texture::BevyDefault,
    },
};

/// Size of the volumetric texture (width, height, depth)
/// Using 128x128x64 for a good balance of quality and performance
pub const VOLUME_SIZE: (u32, u32, u32) = (128, 128, 64);

/// Resource that holds the volumetric texture used for fog, clouds, and other volumetric effects
/// The texture is a 3D RGBA texture where:
/// - RGB channels store the color of the volume
/// - Alpha channel stores the density
#[derive(Resource)]
pub struct VolumetricTexture {
    /// Handle to the 3D texture in Bevy's asset system
    pub texture: Handle<Image>,
}

impl FromWorld for VolumetricTexture {
    fn from_world(world: &mut World) -> Self {
        let mut images = world.resource_mut::<Assets<Image>>();
        
        // Create 3D texture for volumetric data
        let size = Extent3d {
            width: VOLUME_SIZE.0,
            height: VOLUME_SIZE.1,
            depth_or_array_layers: VOLUME_SIZE.2,
        };

        let mut volume_texture = Image::new_fill(
            size,
            TextureDimension::D3,
            &[0, 0, 0, 0],
            TextureFormat::Rgba8Unorm,
        );

        // Configure texture usage flags:
        // - TEXTURE_BINDING: Allow sampling in shaders
        // - COPY_DST: Allow updating the texture data
        // - STORAGE_BINDING: Allow using as a storage texture in compute shaders
        volume_texture.texture_descriptor.usage = TextureUsages::TEXTURE_BINDING 
            | TextureUsages::COPY_DST 
            | TextureUsages::STORAGE_BINDING;

        let texture = images.add(volume_texture);

        Self { texture }
    }
}

/// System that updates the volumetric texture data each frame
/// Creates animated volumetric effects like fog or clouds using noise functions
/// 
/// # Arguments
/// * `images` - Asset storage for accessing the texture data
/// * `volume_texture` - The volumetric texture resource
/// * `time` - Time resource for animation
pub fn update_volume_texture(
    mut images: ResMut<Assets<Image>>,
    volume_texture: Res<VolumetricTexture>,
    time: Res<Time>,
) {
    if let Some(texture) = images.get_mut(&volume_texture.texture) {
        let data = texture.data.as_mut_slice();
        let size = VOLUME_SIZE;
        let t = time.elapsed_seconds();

        // Update volume texture data
        for z in 0..size.2 {
            for y in 0..size.1 {
                for x in 0..size.0 {
                    let index = ((z * size.1 * size.0 + y * size.0 + x) * 4) as usize;
                    
                    // Convert coordinates to normalized space (0 to 1)
                    let pos = Vec3::new(
                        x as f32 / size.0 as f32,
                        y as f32 / size.1 as f32,
                        z as f32 / size.2 as f32
                    );
                    
                    // Create animated fog/clouds using sine waves
                    // Reason: Using simple trigonometric functions for smooth, continuous animation
                    // that creates a natural-looking flow in the volumetric effect
                    let noise = (pos.x * 4.0 + t * 0.1).sin() 
                        * (pos.y * 4.0 + t * 0.2).cos() 
                        * (pos.z * 4.0 + t * 0.15).sin();
                    
                    let density = (noise * 0.5 + 0.5).max(0.0).min(1.0);
                    
                    // RGBA: RGB for color (light gray), A for density
                    data[index] = (255.0 * 0.8) as u8;     // R: Light gray
                    data[index + 1] = (255.0 * 0.9) as u8; // G: Slightly brighter
                    data[index + 2] = 255;                 // B: Full blue component
                    data[index + 3] = (density * 255.0) as u8; // A: Animated density
                }
            }
        }
    }
} 