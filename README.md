# Vampire RPG: The First Immortal

A cross-platform 2D vampire RPG built with Rust and Macroquad, featuring pixel art graphics and atmospheric night environments. Runs identically on Windows, macOS, and Linux. Inspired by the concept of being the original vampire surviving through different epochs of human history.

## Story

You are the sole survivor of a viral outbreak during the Miocene Epoch that transformed early humans into blood-thirsty savages. Unlike the others, the virus affected you differently - you became stronger, faster, and immortal, but with an insatiable thirst for blood and vulnerability to sunlight. As the original vampire, you must survive, adapt, and eventually rule or unite the savage clans that emerged from the infected.

## Game Phases

The game progresses through four distinct phases:

### Phase 1: Survival and Discovery
- **Objective**: Learn your vampire nature and survive the hostile world
- **Gameplay**: Explore, feed on blood sources, avoid sunlight, discover abilities
- **Key Mechanics**: Blood management, day/night cycle, basic combat

### Phase 2: Clan Encounters  
- **Objective**: Make contact with and influence the three main clans
- **Gameplay**: Diplomacy, combat, or deception to gain followers
- **Clans**:
  - **Bone-Eaters**: Brute force specialists (Leader: Grimjaw)
  - **Flame-Haters**: Shadow-dwelling pyrophobes (Leader: Shadowmere)
  - **Night-Bloods**: Stealth hunters (Leader: Silentfang)

### Phase 3: Empire Building
- **Objective**: Build your vampire empire and unite the clans
- **Gameplay**: Territory control, resource management, clan politics

### Phase 4: World Reaction
- **Objective**: Face the consequences of your actions and choose your legacy
- **Gameplay**: Deal with hunters, uncover ancient truths, transcend mortality

## Controls

- **WASD**: Move your character
- **R**: Feed on nearby entities (restores blood and health)
- **E**: Interact with clan leaders (increases trust)
- **Space**: Attack hostile infected enemies
- **Tab**: Toggle clan relations menu
- **L**: Toggle legend (shows what all entities mean)
- **H**: Toggle quick start guide
- **Esc**: Pause/unpause game
- **Ctrl+Q**: Quit game

## Core Mechanics

### Blood System
- **Blood Meter**: Your primary resource that drains over time
- **Feeding**: Approach weak enemies or animals and press R to feed
- **Starvation**: Low blood causes health damage and ability penalties
- **Benefits**: Feeding improves your vampire abilities over time

### Day/Night Cycle
- **Daylight Damage**: Being in sunlight during day causes continuous damage
- **Movement Penalty**: Reduced movement speed during daylight hours
- **Time Display**: Current time and day counter shown in UI
- **Strategic Planning**: Plan activities around the night cycle

### Vampire Abilities
Abilities improve through feeding and experience:
- **Strength**: Increases combat effectiveness
- **Speed**: Improves movement speed
- **Blood Sense**: Enhanced detection of blood sources
- **Shadow Movement**: Advanced stealth capabilities (future feature)

### Visual Guide
The game features beautiful pixel art graphics with a comprehensive legend system:
- **Player**: Detailed vampire sprite with red cape and glowing eyes
- **Clan Leaders**: Pixel warriors with golden crowns and weapons
- **Hostile Infected**: Twisted creatures with red eyes and claws
- **Animals**: Cute pixel animals with ears and tails (blood sources)
- **Environment**: Starry night sky with a glowing moon
- **Effects**: Blood particle effects when feeding
- **Health Bars**: Color-coded bars above all entities (green/yellow/red)
- **Camera**: 1.5x zoom level for enhanced pixel art detail visibility

### Clan Relations
- **Trust System**: Build trust through positive interactions
- **Alliance**: Clans become allied when trust is high enough
- **Reputation**: Your actions affect how all clans perceive you

## Getting Started

### Prerequisites
- Rust 1.70 or higher
- Cargo package manager

### Installation and Running

**On Linux/macOS:**
1. Clone or download the project
2. Navigate to the vampire directory
3. Run the game:
```bash
cargo run
```

**Cross-Compile for Windows (from Linux):**
1. Install Windows target and cross-compiler:
```bash
rustup target add x86_64-pc-windows-gnu
sudo apt install gcc-mingw-w64-x86-64  # Ubuntu/Debian
```
2. Build Windows executable:
```bash
cargo build --release --target x86_64-pc-windows-gnu
```
3. Find the `.exe` at: `target/x86_64-pc-windows-gnu/release/vampire-rpg.exe`

**On Windows:**
1. Install Rust from https://rustup.rs/
2. Navigate to the vampire directory
3. Run: `cargo run`

For optimized performance (all platforms):
```bash
cargo run --release
```

**Cross-Compilation:** See `CROSS_COMPILE.md` for detailed instructions, or run `./build-all.sh` to build both Linux and Windows versions.

## Architecture Overview

The game uses a simplified entity-component system with the following key structures:

### Core Components
- **Position/Velocity**: Movement and physics
- **Health**: Life points for all entities
- **BloodMeter**: Vampire-specific resource system
- **VampireAbilities**: Progressive power system

### Game Systems
- **TimeSystem**: Manages day/night cycles
- **BloodSystem**: Handles feeding, starvation, and sunlight damage
- **MovementSystem**: Updates entity positions
- **InteractionSystem**: Manages player interactions with NPCs

### Entity Types
- **Player**: The vampire protagonist
- **ClanLeader**: Important NPCs that can be allied with
- **HostileInfected**: Enemies that can be fed upon
- **Animal**: Passive blood sources

## Development Roadmap

### Completed Features
- ✅ Basic movement and controls
- ✅ Day/night cycle with sunlight damage
- ✅ Blood feeding system with ability progression
- ✅ Clan interaction and trust system
- ✅ Objective tracking with dynamic completion
- ✅ Comprehensive UI and menus
- ✅ Combat system with hostile infected enemies
- ✅ AI system for enemy behavior
- ✅ Kill and feeding statistics tracking
- ✅ Attack cooldown system
- ✅ In-game legend system (press L)
- ✅ Quick start guide for new players (press H)
- ✅ Pixel art graphics for all entities
- ✅ Atmospheric starry night sky with moon
- ✅ Blood particle effects for feeding
- ✅ Health bars above all entities
- ✅ Enhanced visual feedback and polish

### In Progress Features
- 🔄 Territory control and expansion
- 🔄 Advanced vampire abilities (mist form, wings, shadow command)
- 🔄 Phase transition events and hibernation
- 🔄 Enhanced clan member spawning

### Planned Features
- 📋 Save/load game functionality
- 📋 Audio and improved graphics
- 📋 Hunter enemies in later phases
- 📋 Multiple ending paths
- 📋 Inventory and item system

### Future Enhancements
- 📋 Procedural world generation
- 📋 Dynamic clan member spawning
- 📋 Advanced AI behaviors
- 📋 Resource economy system
- 📋 Ancient vampire lore discoveries

## Code Structure

```
src/
├── main.rs              # Complete game implementation
└── story                # Original design document

assets/                  # Future asset directory
Cargo.toml              # Cross-platform dependencies and targets
README.md               # This file
DEVELOPMENT.md          # Developer guide
CROSS_COMPILE.md        # Cross-compilation guide (Linux → Windows)
build-all.sh            # Cross-platform build script
```

## Contributing

This is a learning project demonstrating Rust game development with Macroquad. Feel free to:

1. Fork the repository
2. Add new features or improvements
3. Fix bugs or optimize performance
4. Enhance the UI and user experience
5. Add more content (clans, abilities, story elements)

## Technical Details

### Dependencies
- **macroquad**: 2D game framework
- **serde/serde_json**: Serialization (for future save system)
- **rand**: Random number generation
- **hecs**: Entity Component System (prepared for future use)

### Performance Notes
- The game is designed to run at 60 FPS with smooth pixel art rendering on all platforms
- 1.5x camera zoom provides excellent pixel art detail without performance impact
- Entity rendering is culled when off-screen
- Particle effects are optimized for performance
- Delta time is capped to prevent large jumps during lag
- Windows: Use `cargo run --release` for best performance
- All platforms: Identical gameplay experience and visual quality

### Cross-Platform Building
- **Native Linux**: Standard `cargo build`
- **Cross-compile to Windows**: `cargo build --release --target x86_64-pc-windows-gnu`
- **Build Both**: Run `./build-all.sh` to create both Linux and Windows executables
- **Self-contained**: Windows `.exe` has no external dependencies (895KB)
- **Identical experience**: Same graphics and gameplay across platforms

### Design Patterns
- Entity-Component-System architecture (simplified)
- State machine for game phases
- Resource management for global game state
- Event-driven interactions

## Gameplay Tips

1. **Start with the Guide**: Press H to see the quick start guide when you begin
2. **Learn the Legend**: Press L to see what all the pixel art characters mean
3. **Enjoy the Atmosphere**: Beautiful starry night sky with twinkling stars and moon
4. **Appreciate the Art**: Game is zoomed in to showcase detailed pixel art sprites
5. **Night is Your Friend**: Plan major activities during nighttime to avoid sunlight damage
6. **Feed Regularly**: Keep your blood meter above 20% to avoid starvation damage
7. **Combat Strategy**: Use Space to attack hostile infected (red-eyed creatures with claws)
8. **Visual Cues**: Look for gold crowns on clan leaders, cute animals for blood sources
9. **Watch the Effects**: Blood particles appear when feeding for visual feedback
10. **Health Monitoring**: Watch the health bars above entities to assess threats
11. **Build Trust Slowly**: Interact repeatedly with clan leaders to build alliances
12. **Progressive Power**: Feeding improves your vampire abilities over time
13. **Survival Focus**: Your first week is about learning the mechanics and staying alive

## License

This project is open source and available for educational and personal use. Feel free to modify and extend as needed.

## Acknowledgments

Inspired by classic vampire fiction and the concept of evolutionary survival horror. Built with Rust and Macroquad to demonstrate game development patterns and vampire RPG mechanics.