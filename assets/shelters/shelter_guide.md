# Shelter System Guide - Vampire RPG

## Overview
The shelter system provides protection from deadly sunlight during daytime. Different shelter types offer varying levels of protection, capacity, and visual appeal.

## Shelter Types & ASCII Art Representations

### Cave (90% Protection)
```
    /^^^^\
   /      \
  |  ● ●   |  <- Cave entrance with stalactites
  |   ___  |     Dark interior provides excellent protection
   \      /
    \____/
```
- **Protection**: 90% (Excellent)
- **Capacity**: 3 entities
- **Condition**: Usually Good to Pristine
- **Discovery Range**: 40m
- **Description**: Natural caves with excellent sunlight blocking

### Underground Bunker (100% Protection)
```
[████████████]
[█ ═══════ █]  <- Metal hatch with grating
[█ ═══════ █]     Perfect protection underground
[█ ═══════ █]
[█ ═══════ █]
[████████████]
   |||||||||     <- Ladder down
```
- **Protection**: 100% (Perfect)
- **Capacity**: 5 entities
- **Condition**: Pristine to Good
- **Discovery Range**: 30m
- **Description**: Emergency bunkers with complete sunlight protection

### Building (80% Protection)
```
    /\  /\  /\
   /  \/  \/  \   <- Roof
  |■ □ ■ □ ■ □|   ■ = walls, □ = windows
  |           |
  | ┌───────┐ |   <- Door
  | |       | |
  |_|_______|_|
```
- **Protection**: 80% (Very Good)
- **Capacity**: 8 entities
- **Condition**: Variable
- **Discovery Range**: 60m
- **Description**: Abandoned buildings with solid walls

### Ancient Ruins (70% Protection)
```
   ╔═══╗  ╔╗
   ║   ║  ║║    <- Partially collapsed walls
   ║ ○ ╚══╝║       Ancient archway entrance
   ║       ║    ● ● ● <- Rubble and debris
   ╚═╗   ○ ║
     ╚═════╝
```
- **Protection**: 70% (Good)
- **Capacity**: 4 entities
- **Condition**: Usually Damaged
- **Discovery Range**: 50m
- **Description**: Historical ruins with atmospheric cover

### Bridge Underpass (75% Protection)
```
████████████████████
║                   ║  <- Bridge structure above
║ │             │   ║     Support pillars
║ │    SAFE     │   ║     Shadowed underpass area
║ │   HAVEN     │   ║
║ │             │   ║
▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀
```
- **Protection**: 75% (Good)
- **Capacity**: 6 entities
- **Condition**: Good to Damaged
- **Discovery Range**: 55m
- **Description**: Urban infrastructure providing overhead cover

### Shed (60% Protection)
```
   ┌─────────┐
  /           \    <- Simple slanted roof
 /             \
│   ┌─────┐   │   <- Small window
│   │     │   │
│   └─────┘   │   ┌──┐ <- Door
│             │   │  │
└─────────────┘   └──┘
```
- **Protection**: 60% (Medium)
- **Capacity**: 2 entities
- **Condition**: Variable
- **Discovery Range**: 45m
- **Description**: Small utility buildings

### Tree Cover (40% Protection)
```
      🌲🌲🌲
    🌲🌲🌲🌲🌲    <- Dense canopy
   🌲🌲🌲🌲🌲🌲
      |||||||      <- Tree trunks
   ░░░SHADE░░░     <- Shaded area below
      |||||||
```
- **Protection**: 40% (Limited)
- **Capacity**: 2 entities
- **Condition**: Good to Damaged
- **Discovery Range**: 35m
- **Description**: Dense foliage providing partial shade

## Shelter Conditions

### Visual Indicators
- **Green Circle**: Pristine condition (100% effectiveness)
- **Yellow Circle**: Good condition (90% effectiveness)
- **Orange Circle**: Damaged condition (70% effectiveness)
- **Red Circle**: Poor condition (40% effectiveness)
- **Gray Circle**: Ruined condition (0% effectiveness)

### Protection Bar
```
████████░░  <- 80% protection (green = good)
██████░░░░  <- 60% protection (yellow = medium)
███░░░░░░░  <- 30% protection (red = poor)
```

## Controls & Mechanics

### Basic Controls
- **F Key**: Enter/Exit nearest shelter
- **Movement**: WASD keys (slower movement in sunlight)
- **Discovery**: Get close to shelters to discover them

### Strategic Considerations

#### Daytime Strategy
1. **Before Dawn**: Seek high-protection shelter
2. **During Day**: Avoid movement unless in shelter
3. **Sunlight Intensity**: Peaks at noon (100% damage)
4. **Protection Needed**: 80%+ for dangerous periods

#### Shelter Selection
- **Emergency**: Any available shelter is better than none
- **Planning**: High-capacity shelters for group protection
- **Long-term**: Pristine condition shelters last longer

#### NPC Behavior
- **Vampires**: Automatically seek shelter during dangerous sunlight
- **Animals**: Unaffected by sunlight
- **Clan Members**: Will compete for shelter space

## Pixel Art Color Palette

### Cave System
- Primary: `#664C33` (Brown)
- Secondary: `#332619` (Dark Brown)
- Entrance: `#191919` (Black interior)

### Building System
- Primary: `#999999` (Gray)
- Secondary: `#4C4C66` (Blue-Gray)
- Details: `#664C19` (Brown door), `#19194C` (Blue windows)

### Tree Cover
- Primary: `#334C33` (Dark Green)
- Secondary: `#194C19` (Darker Green)
- Trunks: `#664433` (Brown)

### Underground
- Primary: `#4C4C4C` (Dark Gray)
- Secondary: `#191919` (Black)
- Metal: `#666666` (Medium Gray)

### Ruins
- Primary: `#7F664C` (Tan)
- Secondary: `#B39966` (Light Tan)
- Debris: `#4C3319` (Dark Brown)

## Implementation Details

### Component Architecture
```rust
// Shelter component with type-specific properties
pub struct Shelter {
    shelter_type: ShelterType,
    condition: ShelterCondition,
    discovered: bool,
    occupants: Vec<u32>,
    // ... additional fields
}

// Entity occupancy tracking
pub struct ShelterOccupancy {
    shelter_id: Option<u32>,
    entered_at: f32,
    seeking_shelter: bool,
}
```

### System Integration
- **Blood System**: Reduces sunlight damage based on shelter protection
- **AI System**: NPCs automatically seek shelter during dangerous periods
- **Rendering**: Pixel art drawing with status indicators
- **UI**: Real-time shelter information and protection status

## Tips for Players

1. **Early Game**: Prioritize discovering nearby shelters
2. **Time Management**: Plan activities around daylight hours
3. **Shelter Quality**: Repair damaged shelters when possible
4. **Capacity Planning**: Share shelters strategically with allies
5. **Emergency Backup**: Always know multiple shelter locations

## Future Enhancements

- **Shelter Building**: Construct custom shelters
- **Shelter Upgrades**: Improve protection and capacity
- **Weather Effects**: Rain/storms affecting shelter integrity
- **Hidden Shelters**: Secret passages and concealed entrances
- **Shelter Networks**: Connected underground systems