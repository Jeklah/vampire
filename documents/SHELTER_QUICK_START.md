# Shelter System Quick Start Guide

## Getting Started

The shelter system is now fully integrated into the Vampire RPG! This guide will help you test and experience all the shelter features.

## How to Run the Game

1. **Build and run**:
   ```bash
   cd vampire
   cargo run
   ```

2. **Alternative**: Build first, then run:
   ```bash
   cargo build
   ./target/debug/vampire-rpg
   ```

## Controls

### Basic Movement
- **WASD**: Move player (vampire)
- **Mouse**: Look around (if implemented)

### Shelter System Controls
- **F Key**: Enter/Exit nearest shelter
- **Movement**: Get close to structures to discover them

### Other Controls
- **R**: Feed on nearby entities
- **Space**: Attack nearby hostiles
- **E**: Interact with clan members
- **Tab**: Toggle clan menu
- **L**: Toggle legend/help
- **H**: Toggle quick start guide
- **Esc**: Pause game
- **Ctrl+Q**: Quit game

## Testing the Shelter System

### 1. Shelter Discovery
- **Walk around the map** to discover the 18 different shelters
- Look for distinctive pixel art structures:
  - Brown cave entrances with rocky details
  - Gray buildings with windows and doors
  - Green tree clusters with dense canopy
  - Metal underground hatches with grating
  - Tan ruins with collapsed walls and debris
  - Brown sheds with slanted roofs
  - Bridge underpasses with support pillars

### 2. Time and Sunlight
- **Check the time display** in the top-left UI
- **Day/Night indicator** shows current time period
- **Sunlight is dangerous** during daytime (6 AM - 6 PM)
- **Peak danger** occurs at noon (12:00) with 100% sunlight intensity

### 3. Shelter Interaction
- **Approach any structure** to see if it's a shelter
- **Press F** when near a discovered shelter to enter
- **Check UI** for shelter status and protection level
- **Press F again** while inside to exit

### 4. Protection System
- **Monitor health** during daytime outside vs inside shelters
- **Watch for exposure warnings** in the UI
- **Different shelters** provide different protection levels:
  - Underground Bunker: 100% protection (perfect)
  - Cave: 90% protection (excellent)
  - Building: 80% protection (very good)
  - Bridge Underpass: 75% protection (good)
  - Ruins: 70% protection (good)
  - Shed: 60% protection (medium)
  - Tree Cover: 40% protection (limited)

### 5. Capacity Testing
- **Enter a shelter** and note the occupancy count
- **Different shelter types** have different capacities:
  - Building: 8 entities (largest)
  - Underground: 5 entities
  - Ruins: 4 entities
  - Cave: 3 entities
  - Shed: 2 entities (smallest)
  - Tree Cover: 2 entities
  - Bridge Underpass: 6 entities

### 6. NPC Behavior
- **Watch NPCs** during dangerous sunlight periods
- **Vampire NPCs** will automatically seek shelter when sunlight > 60%
- **Competition** may occur for limited shelter space
- **Animals** are unaffected by sunlight

### 7. UI Information
- **Nearby Shelters** section shows discovered shelters within 200m
- **Distance indicators** help you find the closest options
- **Protection status** shows if you're currently protected
- **Shelter descriptions** include protection level and capacity

## What to Look For

### Visual Elements
- **Unique pixel art** for each shelter type
- **Status indicators**:
  - Green/Yellow/Red protection bars
  - Blue occupancy indicators when shelters are occupied
  - Green/Yellow/Orange/Red condition circles
- **Atmospheric details** like debris, vegetation, shadows

### Gameplay Elements
- **Strategic movement** planning around daylight cycles
- **Emergency shelter seeking** when caught in dangerous sunlight
- **Resource competition** with NPCs for shelter space
- **Discovery exploration** to find better shelters

### Technical Features
- **Smooth integration** with existing game systems
- **Real-time protection calculations** affecting health
- **Dynamic AI behavior** responding to environmental threats
- **Performance optimization** with efficient rendering

## Shelter Locations

The game world (1600x800) contains 18 strategically placed shelters:

### High Protection Shelters
- **Emergency Bunker** (350, 600) - Underground, 100% protection
- **Deep Cavern** (800, 100) - Cave, 90% protection
- **Ancient Cave** (200, 150) - Cave, 90% protection

### Medium Protection Shelters
- **Abandoned House** (500, 300) - Building, 80% protection
- **Old Warehouse** (700, 500) - Building, 80% protection
- **Highway Underpass** (600, 400) - Bridge, 75% protection

### Lower Protection Shelters
- **Dense Grove** (300, 500) - Tree Cover, 40% protection
- Multiple sheds and damaged structures scattered throughout

## Troubleshooting

### Common Issues
1. **Can't find shelters**: Walk closer to structures, discovery range varies by type
2. **Can't enter shelter**: Check if at capacity or in poor condition
3. **Still taking damage**: Verify shelter protection level vs sunlight intensity
4. **No shelter interaction**: Make sure you're within discovery range and press F

### Performance
- Game should run smoothly with minimal performance impact
- Shelter rendering is optimized with viewport culling
- All 18 shelters can be active simultaneously without issues

## Advanced Testing

### Stress Testing
- **Fill shelters to capacity** with multiple NPCs
- **Test during peak sunlight** (noon) with different shelter types
- **Rapid entry/exit** to test state management
- **Long-term occupation** to test time tracking

### Edge Cases
- **Shelter destruction** (if implemented in future)
- **Multiple nearby shelters** and selection priority
- **Shelter discovery** at maximum zoom levels
- **Performance** with many entities seeking shelter simultaneously

## Development Features

### Debug Information
- Enable debug rendering to see protection percentages
- Debug messages show shelter interaction results
- Console output for detailed system state

### Testing Commands
- Run `cargo test shelter` for unit tests
- Use `cargo run --example shelter_usage` for system examples
- Check `cargo doc --open` for detailed API documentation

## Next Steps

After testing the basic functionality, consider:
1. **Exploring all shelter types** and comparing their effectiveness
2. **Strategic gameplay** planning movement around time cycles
3. **NPC interaction** observing AI shelter seeking behavior
4. **Performance testing** with extended gameplay sessions

The shelter system provides a solid foundation for survival gameplay while maintaining the atmospheric pixel art aesthetic of the vampire RPG. Enjoy exploring this new survival mechanic!