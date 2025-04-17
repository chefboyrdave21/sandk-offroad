# SandK Offroad - Rust Edition 🚙

A high-performance offroad racing game built with Rust, featuring advanced physics, dynamic terrain, immersive gameplay, and comprehensive content creation tools.

## 🎮 Features

### Core Systems
- **Physics System**: Advanced vehicle physics with realistic handling and terrain interaction
- **Terrain System**: Dynamic terrain generation with:
  - Procedural heightmap generation
  - Terrain blending and transitions
  - Deformation and destruction
  - Erosion and weathering effects
  - Path creation and management
  - LOD (Level of Detail) optimization
  - Chunk-based terrain loading
  - Advanced texturing and material system

### Vehicle Systems
- **Physics Integration**: Realistic vehicle dynamics
- **Advanced Suspension**: 
  - Air ride functionality with 10ft lift capability
  - Long arm suspension options
  - Customizable spring rates and damping
  - Real-time suspension telemetry
- **Control System**: Precise steering, acceleration, and braking
- **Damage System**: Vehicle deformation and damage modeling
- **Statistics Tracking**: Performance metrics and vehicle stats
- **Input Handling**: Customizable controls and input mapping

### Visual Systems
- **Camera System**:
  - Third Person View
  - Wheel View
  - First Person View (FPV)
  - Side View
  - Dynamic camera transitions
  - Spectator mode with multi-vehicle tracking
  - Broadcasting camera tools
- **Particle System**: Visual effects for:
  - Dust and dirt
  - Water splashes
  - Vehicle exhaust
  - Weather effects
- **Weather System**: Dynamic weather conditions with:
  - Rain
  - Snow
  - Fog
  - Dynamic lighting

### Audio System
- 3D positional audio
- Environmental sound effects
- Vehicle engine sounds
- Weather audio effects
- CB Radio communication with:
  - Distance-based signal degradation
  - Multiple channels
  - Team communication features
  - Music broadcasting capabilities
  - Stream buffering and quality controls
- In-Game Radio Stations:
  - Off-Road Radio (Rock/Metal)
  - Trail Ambient
  - Adventure Beats
  - Garage Grooves
  - Competition Mix
  - Custom Team Radio stations
- Advanced Audio Features:
  - Local music streaming to team
  - Collaborative playlist management
  - DJ role system
  - Smart audio mixing
  - Cross-platform audio sharing
  - Voice-over capabilities

### Content Creation & Sharing
- **Video Capture System**:
  - Multi-quality recording presets (720p/30fps to 4K/60fps)
  - Hardware-accelerated encoding (NVENC, AMF, QuickSync)
  - Buffer-based recording system
  - Multiple audio track support
  - Performance-optimized capture engine
- **Live Streaming Integration**:
  - Multi-platform streaming (Twitch, YouTube, Rumble)
  - Custom overlay system with vehicle telemetry
  - Stream management tools
  - Chat integration and viewer commands
  - Stream archive system
- **Photo Mode**:
  - High-resolution screenshot capture
  - Custom filters and effects
  - Social media integration
  - Community showcase features

### Game Systems
- **Resource Management**: Efficient asset loading and caching
- **Scene Management**: Dynamic scene loading and optimization
- **UI System**: 
  - HUD (Heads-Up Display)
  - Menu system
  - In-game notifications
- **Mission System**: Objectives and rewards
- **AI System**: Computer-controlled opponents
- **Networking**: Multiplayer support
- **Social Features**:
  - Team formation system
  - Global chat
  - Vehicle marketplace
  - Trail rating system
  - Community events

## 🚧 Project Status
Currently implementing core features with the following progress:

### Completed Tasks (0/38)
- Initial project setup and core systems in progress

### In Progress
- Core engine development
- Physics system implementation
- Terrain generation system
- Advanced suspension system
- Video capture and streaming integration
- Spectator system development
- Social features implementation

### Next Planned Features
- Multiplayer support with dedicated servers
- Additional vehicle types and customization
- Expanded weather conditions
- Track editor
- Vehicle customization system
- Online leaderboards

## 🛠️ Technical Details
- Built with Rust for maximum performance
- Advanced physics simulation
- Optimized rendering pipeline
- Cross-platform support
- Modern graphics features

## 🚀 Getting Started

### Prerequisites

#### Linux
```bash
# Install Rust
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# Install system dependencies
# Ubuntu/Debian
sudo apt-get update
sudo apt-get install -y build-essential pkg-config libasound2-dev libudev-dev

# Fedora
sudo dnf install -y gcc pkg-config alsa-lib-devel systemd-devel

# Arch Linux
sudo pacman -S base-devel pkg-config alsa-lib systemd

# Manjaro Linux
# Enable AUR if not already enabled
sudo pacman -S --needed base-devel git
sudo pacman -S pkg-config alsa-lib systemd
# Install additional dependencies for graphics
sudo pacman -S vulkan-icd-loader vulkan-validation-layers
# For NVIDIA users
sudo pacman -S nvidia-utils lib32-nvidia-utils
# For AMD users
sudo pacman -S vulkan-radeon lib32-vulkan-radeon
# For Intel users
sudo pacman -S vulkan-intel lib32-vulkan-intel
```

#### Windows
1. Install Rust using [rustup](https://rustup.rs/)
2. Install [Visual Studio Build Tools](https://visualstudio.microsoft.com/visual-cpp-build-tools/)
3. Install [LLVM](https://github.com/llvm/llvm-project/releases) and add it to your PATH

#### Docker
```bash
# Build the Docker image
docker build -t sandk-offroad:latest .

# Run the container
docker run -it sandk-offroad:latest
```

### Building from Source

#### Linux/Windows
```bash
# Clone the repository
git clone https://github.com/yourusername/sandk-offroad.git
cd sandk-offroad

# Build in debug mode
cargo build

# Build in release mode
cargo build --release
```

#### Docker
```bash
# Build the Docker image
# First, log in to Docker Hub (use environment variables or Docker credentials store)
docker login

# Build and push the image
docker build -t yourusername/sandk-offroad:latest .
docker push yourusername/sandk-offroad:latest
```

### Docker Security Best Practices

#### Secure Credential Management
```bash
# Option 1: Use environment variables (recommended for CI/CD)
export DOCKER_USERNAME=yourusername
export DOCKER_PASSWORD=yourtoken  # Use a Personal Access Token, not your password
docker login -u $DOCKER_USERNAME -p $DOCKER_PASSWORD

# Option 2: Use Docker credential store (recommended for local development)
# This will securely store credentials in your system's keychain
docker login  # Enter credentials once, they'll be stored securely

# Option 3: Use credential helper (platform specific)
# Linux: secretservice
sudo apt-get install pass gnupg2
pass init "Your GPG ID"
docker-credential-pass install

# macOS: osxkeychain
docker-credential-osxkeychain install

# Windows: wincred
# Already configured with Docker Desktop
```

#### Environment File for Credentials
Create a `.env` file (add to .gitignore):
```bash
# .env
DOCKER_USERNAME=yourusername
DOCKER_TOKEN=yourtoken
DOCKER_REGISTRY=registry.example.com
```

Load environment variables:
```bash
# Load variables before building/pushing
source .env
docker login -u $DOCKER_USERNAME -p $DOCKER_TOKEN $DOCKER_REGISTRY
```

#### CI/CD Security
For GitHub Actions:
```yaml
# .github/workflows/docker.yml
jobs:
  build:
    steps:
      - uses: actions/checkout@v2
      - name: Login to Docker Hub
        uses: docker/login-action@v1
        with:
          username: ${{ secrets.DOCKER_USERNAME }}
          password: ${{ secrets.DOCKER_TOKEN }}
```

For GitLab CI:
```yaml
# .gitlab-ci.yml
docker_build:
  script:
    - docker login -u $CI_REGISTRY_USER -p $CI_REGISTRY_PASSWORD $CI_REGISTRY
```

#### Additional Security Measures
- Use Personal Access Tokens instead of passwords
- Regularly rotate credentials
- Use minimal scope for access tokens
- Never commit credentials to version control
- Use multi-stage builds to minimize attack surface
- Scan images for vulnerabilities before pushing
- Use signed images in production

### Running the Game

#### Linux/Windows
```bash
# Run in debug mode
cargo run

# Run in release mode
cargo run --release
```

#### Docker
```bash
# Run with X11 forwarding (Linux)
docker run -it --rm \
  -e DISPLAY=$DISPLAY \
  -v /tmp/.X11-unix:/tmp/.X11-unix \
  sandk-offroad

# Run with GPU acceleration (Linux)
docker run -it --rm \
  --gpus all \
  -e DISPLAY=$DISPLAY \
  -v /tmp/.X11-unix:/tmp/.X11-unix \
  sandk-offroad
```

### Controls
- W: Move Backward
- S: Move Forward
- A: Turn Left
- D: Turn Right
- ESC: Pause Menu
- Q: Quit Game

### Configuration
The game can be configured by editing the `config.ron` file in the game's root directory. Available options include:
- Graphics settings
- Audio settings
- Control bindings
- Game difficulty

### Streaming & Content Creation Settings
The following settings can be configured in `config.ron`:

#### Video Capture
```ron
video_capture: (
    // Buffer settings
    buffer_duration_seconds: 300,  // 5 minutes of replay buffer
    buffer_memory_limit_mb: 2048,  // 2GB memory limit
    
    // Quality presets
    quality_presets: {
        "low": (
            resolution: (1280, 720),
            fps: 30,
            bitrate_mbps: 5,
        ),
        "medium": (
            resolution: (1920, 1080),
            fps: 60,
            bitrate_mbps: 10,
        ),
        "high": (
            resolution: (3840, 2160),
            fps: 60,
            bitrate_mbps: 30,
        ),
    },
    
    // Hardware encoding
    preferred_encoders: ["NVENC", "AMF", "QuickSync", "x264"],
    enable_gpu_acceleration: true,
)
```

#### Streaming Platforms
```ron
streaming: (
    // Platform credentials (DO NOT commit these to version control!)
    twitch: (
        client_id: "your_client_id",
        client_secret: "your_client_secret",
        stream_key: "your_stream_key",
    ),
    youtube: (
        api_key: "your_api_key",
        stream_key: "your_stream_key",
    ),
    rumble: (
        api_key: "your_api_key",
        stream_key: "your_stream_key",
    ),

    // Stream settings
    default_platform: "twitch",
    auto_start_stream: false,
    chat_overlay: true,
    show_viewer_count: true,
    
    // Overlay configuration
    overlay: (
        show_telemetry: true,
        show_trail_info: true,
        show_team_status: true,
        custom_overlay_path: "assets/overlays/default.ron",
    ),
)
```

#### Audio Mixing
```ron
audio_mixing: (
    // Channel volumes (0.0 to 1.0)
    game_audio: 0.8,
    voice_chat: 1.0,
    cb_radio: 0.9,
    music: 0.5,

    // Audio devices
    input_device: "default",
    output_device: "default",
    
    // Voice activation
    voice_activation_threshold: -40.0,  // dB
    noise_gate_threshold: -45.0,        // dB
)
```

#### WebRTC Configuration
```ron
webrtc: (
    // Signaling server configuration
    signaling: (
        default_server: "wss://signaling.sandk-offroad.com",
        custom_server: "",  // Set to use your own signaling server
        reconnect_interval_ms: 3000,
        max_reconnect_attempts: 5,
    ),

    // STUN/TURN configuration
    ice_servers: [
        // Default public STUN servers
        {
            urls: ["stun:stun.l.google.com:19302", "stun:stun1.l.google.com:19302"],
        },
        // Custom TURN server (self-hosted)
        {
            urls: ["turn:your-turn-server.com:3478"],
            username: "your_username",
            credential: "your_password",
        },
    ],

    // Coturn server settings (for self-hosting)
    coturn: (
        enable: false,
        server_url: "turn:your-server:3478",
        realm: "sandk-offroad.com",
        username: "your_username",
        credential: "your_password",
        // Advanced settings
        min_port: 49152,
        max_port: 65535,
        enable_tcp: true,
        enable_udp: true,
    ),

    // Stream settings
    peer_connection: (
        // Video configuration
        max_bitrate_kbps: 8000,
        min_bitrate_kbps: 1000,
        max_resolution: (3840, 2160),  // 4K
        min_resolution: (640, 360),    // 360p
        preferred_codec: "VP9",
        
        // Connection settings
        ice_transport_policy: "all",   // "all" or "relay"
        bundle_policy: "max-bundle",
        rtcp_mux: true,
        
        // Performance
        enable_simulcast: true,
        enable_bandwidth_estimation: true,
        max_packet_loss_percentage: 2,
    ),

    // Security settings
    security: (
        enable_encryption: true,
        authentication_required: true,
        allowed_domains: ["*"],        // Restrict to specific domains
        max_concurrent_viewers: 10,
    ),
)
```

#### Performance Management
```ron
performance: (
    // Recording impact limits
    max_cpu_usage_percent: 10,
    max_gpu_usage_percent: 15,
    max_memory_usage_gb: 4,
    
    // Auto-quality adjustment
    enable_auto_quality: true,
    quality_check_interval_ms: 1000,
    frame_drop_threshold: 0.02,         // 2% frame drop tolerance
)
```

## 📝 License
This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.

## 🤝 Contributing
1. Fork the repository
2. Create your feature branch (`git checkout -b feature/amazing-feature`)
3. Commit your changes (`git commit -m 'Add some amazing feature'`)
4. Push to the branch (`git push origin feature/amazing-feature`)
5. Open a Pull Request

## 🐛 Known Issues
- Some graphics artifacts on certain hardware configurations
- Occasional physics glitches on complex terrain
- Audio may stutter on lower-end systems

### Manjaro Linux Specific Issues

#### Graphics Issues
If you experience graphics issues on Manjaro:
```bash
# Check Vulkan installation
vulkaninfo

# Install missing Vulkan drivers
# For NVIDIA
sudo mhwd -i pci video-nvidia

# For AMD
sudo mhwd -i pci video-amdgpu

# For Intel
sudo mhwd -i pci video-hybrid-intel
```

#### Audio Issues
If you experience audio issues:
```bash
# Check audio system
systemctl status alsa-state.service

# Restart audio service
sudo systemctl restart alsa-state.service

# Install additional audio packages if needed
sudo pacman -S alsa-utils pulseaudio-alsa
```

#### Docker Issues
If you have issues running the game in Docker:
```bash
# Install Docker dependencies
sudo pacman -S docker docker-compose

# Start Docker service
sudo systemctl start docker
sudo systemctl enable docker

# Add user to docker group
sudo usermod -aG docker $USER

# Install NVIDIA container toolkit (if using NVIDIA GPU)
sudo pacman -S nvidia-container-toolkit
sudo systemctl restart docker
```

## 📞 Support
For support, please open an issue in the GitHub repository or join our Discord community.

## 🙏 Acknowledgments
- Bevy Engine team
- Rapier Physics team
- All contributors and supporters 

## 🎯 Latest Release (v1.0.0)
### Major Changes
- Improved vehicle physics system with realistic mass and momentum
- Enhanced terrain system with dynamic mesh generation
- Reversed vehicle controls for better gameplay feel
- Optimized collision detection and response
- Reduced friction coefficients for smoother movement
- Improved camera following system
- Better ground detection and suspension

### Technical Improvements
- Reduced physics damping for more responsive controls
- Adjusted vehicle mass to 1000kg for better handling
- Optimized terrain mesh generation
- Enhanced collision response system
- Improved frame rate and performance
- Better debug visualization tools

### Docker Support
The game is now available as a Docker image! Pull the latest version:
```bash
docker pull dounoit/sandk-offroad:v1.0.0
```

Run with GPU acceleration:
```bash
docker run -it --rm \
  --gpus all \
  -e DISPLAY=$DISPLAY \
  -v /tmp/.X11-unix:/tmp/.X11-unix \
  dounoit/sandk-offroad:v1.0.0
``` 