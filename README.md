# Vampire RPG: The First Immortal

A 2D vampire RPG built with Rust and Macroquad, inspired by the concept of being the original vampire surviving through different epochs of human history.

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
The game includes a comprehensive legend system:
- **Player**: Red square with white border (you)
- **Clan Leaders**: Colored squares with gold crowns
- **Hostile Infected**: Dark red squares with red X marks
- **Animals**: Brown circles (blood sources)
- **Health Bars**: Color-coded bars above all entities (green/yellow/red)

### Clan Relations
- **Trust System**: Build trust through positive interactions
- **Alliance**: Clans become allied when trust is high enough
- **Reputation**: Your actions affect how all clans perceive you

## Getting Started

### Prerequisites
- Rust 1.70 or higher
- Cargo package manager

### Installation and Running

1. Clone or download the project
2. Navigate to the vampire directory
3. Run the game:
```bash
cargo run
```

For optimized performance:
```bash
cargo run --release
```

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
- ✅ Visual entity distinction with symbols
- ✅ Health bars above all entities
- ✅ Enhanced visual feedback

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
└── story                # Original game design document

assets/                  # Future asset directory
Cargo.toml              # Project dependencies
README.md               # This file
DEVELOPMENT.md          # Developer guide
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
- The game is designed to run at 60 FPS
- Entity rendering is culled when off-screen
- Delta time is capped to prevent large jumps during lag

### Design Patterns
- Entity-Component-System architecture (simplified)
- State machine for game phases
- Resource management for global game state
- Event-driven interactions

## Gameplay Tips

1. **Start with the Guide**: Press H to see the quick start guide when you begin
2. **Learn the Legend**: Press L to see what all the colored shapes mean
3. **Night is Your Friend**: Plan major activities during nighttime to avoid sunlight damage
4. **Feed Regularly**: Keep your blood meter above 20% to avoid starvation damage
5. **Combat Strategy**: Use Space to attack hostile infected (red squares with X marks)
6. **Visual Cues**: Look for gold crowns on clan leaders, circles for animals
7. **Health Monitoring**: Watch the health bars above entities to assess threats
8. **Build Trust Slowly**: Interact repeatedly with clan leaders to build alliances
9. **Progressive Power**: Feeding improves your vampire abilities over time
10. **Survival Focus**: Your first week is about learning the mechanics and staying alive

## License

This project is open source and available for educational and personal use. Feel free to modify and extend as needed.

## Acknowledgments

Inspired by classic vampire fiction and the concept of evolutionary survival horror. Built with Rust and Macroquad to demonstrate game development patterns and vampire RPG mechanics.