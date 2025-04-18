# Asset Management System Documentation

## Overview

The asset management system is responsible for loading, managing, and hot-reloading game assets. It supports prioritized loading, multiple asset types, and provides detailed loading state information.

## Asset Types

The system supports the following asset types:

### Vehicle Assets
- Models (`.gltf`, `.glb`)
- Textures (`.png`, `.jpg`)
- Configurations (`.json`)

### Audio Assets
- Engine sounds (`.ogg`, `.wav`)
- Environment sounds
- Music tracks
- UI sounds
- Radio stations
- Voice lines

### UI Assets
- Textures
- Fonts
- Icons
- Animations

### Terrain Assets
- Textures
- Heightmaps
- Objects

### Effect Assets
- Particle textures
- Weather effects

### Shader Assets
- Custom shaders
- Material definitions

## Loading Priority System

Assets are loaded in the following priority order:

1. **Critical Priority**
   - UI elements required for loading screen
   - Essential fonts
   - Core game systems

2. **High Priority**
   - Vehicle assets
   - Player-related content
   - Main menu assets

3. **Medium Priority**
   - Terrain assets
   - Environment objects
   - Common effects

4. **Low Priority**
   - Audio assets
   - Background content
   - Optional content

## Asset Loading State

The `AssetLoadingState` struct provides detailed information about the loading process:

```rust
pub struct AssetLoadingState {
    pub total_assets: usize,
    pub loaded_assets: usize,
    pub failed_assets: usize,
    pub is_complete: bool,
    pub current_priority: LoadPriority,
    pub loading_queue: Vec<HandleId>,
}
```

## Usage Examples

### Loading Assets

```rust
// Get the GameAssets resource
let game_assets = world.get_resource::<GameAssets>();

// Load all assets
game_assets.load_all(&asset_server);

// Check loading progress
if let Some(loading_state) = world.get_resource::<AssetLoadingState>() {
    println!("Loading progress: {}/{}", 
        loading_state.loaded_assets,
        loading_state.total_assets
    );
}
```

### Hot Reloading (Debug Mode)

```rust
#[cfg(debug_assertions)]
game_assets.hot_reload(&asset_server);
```

## Directory Structure

```
assets/
├── vehicles/
│   ├── models/
│   ├── textures/
│   └── configs/
├── audio/
│   ├── engine/
│   ├── environment/
│   ├── music/
│   ├── ui/
│   ├── radio/
│   └── voice/
├── ui/
│   ├── textures/
│   ├── fonts/
│   └── icons/
├── effects/
│   ├── particles/
│   └── weather/
├── terrain/
│   ├── textures/
│   ├── heightmaps/
│   └── objects/
└── shaders/
    ├── custom/
    └── materials/
```

## Best Practices

1. **Asset Organization**
   - Keep assets in their appropriate directories
   - Use consistent naming conventions
   - Include version numbers in filenames when applicable

2. **Loading Optimization**
   - Prioritize essential assets
   - Use appropriate file formats for each asset type
   - Compress assets when possible

3. **Error Handling**
   - Always check for failed assets
   - Provide fallback assets for critical content
   - Log loading errors appropriately

4. **Hot Reloading**
   - Only enable in debug builds
   - Ensure assets are properly cleaned up
   - Handle state transitions smoothly

## Troubleshooting

Common issues and solutions:

1. **Missing Assets**
   - Check file paths and naming
   - Verify asset directories exist
   - Check file permissions

2. **Loading Failures**
   - Verify file formats
   - Check for corrupted files
   - Monitor memory usage

3. **Performance Issues**
   - Review asset sizes
   - Check loading priorities
   - Monitor loading queue size

## Contributing

When adding new asset types:

1. Update the `GameAssets` struct
2. Add appropriate loading priority
3. Update tests
4. Document the new asset type
5. Update the directory structure if needed 