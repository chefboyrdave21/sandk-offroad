# Particle System Examples

This directory contains example implementations showcasing the particle system's capabilities.

## Basic Particle Example

The basic particle example demonstrates various particle effects and features of the system.

### Running the Example

Add the example plugin to your app:

```rust
use sandk_offroad::game::plugins::particle_system::BasicParticleExamplePlugin;

fn main() {
    App::new()
        .add_plugins((
            DefaultPlugins,
            BasicParticleExamplePlugin,
        ))
        .run();
}
```

### Features Demonstrated

1. **Multiple Effect Types**
   - Fire: Realistic fire effect with dynamic emission
   - Smoke: Billowing smoke with alpha blending
   - Magic: Glowing magical particles with trails
   - Water: Fluid-like particles with refraction
   - Heal: Healing effect with custom colors

2. **Interactive Controls**
   - `Space`: Toggle all effects on/off
   - `1-5`: Spawn different effect types
   - `B`: Toggle effect bounds visualization
   - `P`: Toggle performance metrics display

3. **Performance Monitoring**
   - FPS counter
   - Active effect count
   - Total particle count
   - Real-time performance metrics

4. **Visual Features**
   - Dynamic movement patterns
   - Trail effects for certain particles
   - Soft particles with depth buffer
   - Custom blend modes
   - Emission strength variation

### Implementation Details

The example is structured into several key components:

1. **Plugin Setup** (`BasicParticleExamplePlugin`)
   - Initializes necessary plugins
   - Sets up debug resources
   - Configures systems for updates

2. **Effect Management**
   - `spawn_effect`: Creates particle effects with appropriate configuration
   - `update_emitter_position`: Handles unique movement patterns
   - `handle_input`: Processes user input for spawning and control

3. **Debug Features**
   - Performance metrics display
   - Control reference
   - Visual debugging tools

### Customization

To modify the example:

1. **Adjust Effect Parameters**
   ```rust
   let config = BasicParticleConfig {
       effect_type: BasicParticleEffect::Fire,
       scale: 2.0,
       intensity: 1.5,
       emission_strength: 2.0,
       ..Default::default()
   };
   ```

2. **Add New Movement Patterns**
   ```rust
   match effect_type {
       BasicParticleEffect::Custom => {
           transform.translation = Vec3::new(
               /* Custom movement logic */
           );
       }
   }
   ```

3. **Extend Debug Information**
   ```rust
   if debug_config.show_performance {
       info.push_str(&format!("Custom Metric: {}\n", value));
   }
   ```

### Best Practices

1. **Performance**
   - Monitor particle counts
   - Use LOD when appropriate
   - Enable soft particles only when needed

2. **Visual Quality**
   - Adjust emission rates based on effect type
   - Use appropriate blend modes
   - Consider depth buffer interactions

3. **Memory Management**
   - Clean up inactive effects
   - Limit maximum particles per system
   - Use appropriate buffer sizes

## Contributing

To add new examples:

1. Create a new module in the `examples` directory
2. Implement the example plugin
3. Add documentation
4. Update the `mod.rs` file to expose the new example

## License

This example code is part of the SandK Offroad project and is subject to its licensing terms. 