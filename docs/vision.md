# Multiplayer Fighting Game Vision

Build a small learning-first 1v1 fighting game in Rust with Raylib. The goal is something playable with friends while learning fighting game architecture and networking. Clarity and correctness matter more than polish.

## Core Shape

- One-screen side-view arena
- Flat ground, no scrolling camera
- Two box fighters
- Local two-player keyboard controls first
- Fixed `60 Hz` deterministic simulation
- Raylib handles window, input, rendering, and later audio
- A headless game module handles all gameplay state and rules
- Plain Rust structs and enums, no ECS initially

Target sim shape:

```rust
GameState + PlayerInput -> GameState
```

## First Vertical Slice

Make a local deterministic mini-match:

- movement and jumping
- facing direction
- one standing melee attack
- startup, active, and recovery frames
- hitbox versus hurtbox collision
- hitstun
- health
- reset on defeat

## Networking Direction

After the local core works, add deterministic peer-to-peer lockstep with input delay.

Assume LAN, VPN, or manual IP. No matchmaking, NAT traversal, relay server, or rollback at first.

## Non-Goals For First Slice

- menus
- rounds
- character select
- sprite art
- sound
- multiple characters
- multiple attacks
- combos
- projectiles
- platforms
- camera work
- networking
- ECS

## Milestones

1. Local deterministic core
2. LAN or manual-IP P2P lockstep
3. Tiny game feel pass
4. Optional rollback experiment
