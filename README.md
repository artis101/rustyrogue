# Rustyrogue - A roguelike game written in Rust

Rustyrogue is a roguelike game written in Rust. It is a work in progress.
The goal is to make a simple roguelike game that is easy to understand and modify.
Additionally this game is my learning project for Rust and game development.

![Screenshot of ASCII renderer](https://github.com/artis101/rustyrogue/blob/main/assets/screenshot.png?raw=true "Screenshot of Rustyrogue in an early development stage.")

## Features

This is a non-exhaustive list of features that I plan to implement in this game.

It also serves as a roadmap and a todo list for the project.

- [x] ASCII graphics
- [x] Ratatui for terminal graphics
- [x] Field of view ASCII implementation
- [x] Working map loader
- [x] Basic player movement
- [x] Basic interaction with the environment
  - [x] Opening doors
  - [ ] Proper visibility system with game log messages
  - [ ] Picking up items
  - [ ] Attacking enemies
  - [ ] Using items
  - [ ] Reading scrolls
  - [ ] Reading signs
  - [ ] Using stairs
  - [ ] Using portals/arches
  - [ ] Boss fights/boss rooms
- [ ] Different environment tiles
  - [x] Deadly pits
  - [-] Pressure plates/secret floor tiles
  - [ ] Water tiles
  - [ ] Fire tiles
  - [x] Cursed tiles
  - [ ] Spikes ^ or v
  - [ ] Trap tiles
  - [ ] Revealing tiles ?
  - [x] Obelisks
- [ ] Basic combat system
- [ ] Basic AI for enemies
- [ ] Basic inventory system
- [ ] Making use of turns and time
- [ ] Level generation
  - [ ] Random level generation
  - [ ] Template level generation
  - [ ] Dungeon generation
  - [ ] Infinite level generation with interconnecting rooms
- [ ] Level navigation
  - [ ] Exiting a level through the doors
  - [ ] Going up and down stairs
  - [ ] Going back to the previous level through the same entrance
  - [ ] Game state/explored dungeon map
  - [ ] Minimap
- [ ] SDL2 renderer
  - [ ] Basic rendering
  - [ ] Load and render sprite from files
  - [ ] Implement animations
  - [ ] Implement particle system
  - [ ] Succumb to using bevy or ggez
- [ ] Saving and loading the game

## What is this game about?

Rustyrogue is a treacherous roguelike dungeon crawler.
You wake up in a dungeon with no memory of how you got there.
Around you are walls. You can see an object right in front of you
yet you cannot make out what it is. You can hear the sound of
something moving in the distance. You have to find your way out.

Use weapons, spells, and items to defeat enemies and find your way out.
Dig through the dungeon, find secret rooms, and uncover hidden treasures.

Enjoy the endless possibilities of a procedurally generated dungeon.
Every playthrough is different. Every room is a new challenge.

## Q&A

**Q:** Why Rust?
**A:** Rust is safe, fast and easy to write. I also wanted to learn a systems
programming language. Rust ended up being the best choice for me.

**Q:** Why a roguelike?
**A:** I think a roguelike is a good starting point for a game. Simple enough
to understand, complex enough to be interesting. I also like the ASCII
graphics. It's a nice aesthetic and brings me back to my childhood.
Also _very_ easy to work with.

**Q:** ASCII?
**A:** ASCII graphics are simple to implement and easy to understand.
I can save time on tooling by not making a map editor. I can also
focus on the game mechanics and not the graphics. Adding a few
colors brings the game to life.

**Q:** What state is the game in?
**A**: The game is in early development. I am still working on the basic mechanics.
The todo list is far from complete or comprehensive.
