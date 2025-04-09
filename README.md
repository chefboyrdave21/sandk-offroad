# SandK Offroad - Rust Edition 🚙

A high-performance offroad racing game built with Rust, featuring advanced physics, dynamic terrain, and immersive gameplay.

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

## 🚧 In Progress
- Vegetation system with dynamic placement
- Level progression system
- Achievement system
- Save/Load functionality

## 📋 Planned Features
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
# Install Docker
# Linux
curl -fsSL https://get.docker.com | sh

# Windows
# Download and install Docker Desktop from https://www.docker.com/products/docker-desktop
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
docker build -t sandk-offroad .

# Run the container
docker run -it --rm \
  -e DISPLAY=$DISPLAY \
  -v /tmp/.X11-unix:/tmp/.X11-unix \
  sandk-offroad
```

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
- W: Accelerate
- S: Brake/Reverse
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