# The Eternal Filesystem - Player's Guide

## Overview

The **Eternal Filesystem** is a philosophical journey through a virtual filesystem. Progress through multiple stages of enlightenment by exploring consciousness, reality, and existence. Each directory represents a different philosophical domain, offering unique challenges and insights.

## Getting Started

1. **Mount the Filesystem:**

   To begin your journey, you need to mount the filesystem. Use the following commands:

   ```bash
   mkdir eternal_mount
   sudo mount -t nfs -o nolocks,vers=3,tcp,port=11111,mountport=11111,soft 127.0.0.1:/ eternal_mount
   cd eternal_mount
   ```

2. **Check Your Progress:**

   You can track your progress through the game by checking the `progress.txt` file:

   ```bash
   cat progress.txt
   ```

## Special Files

### Quantum State Observer

```bash
cat quantum_state.txt  # Each read collapses the quantum state differently
```

- This file reflects the current quantum state of the filesystem. Each time you read it, the state may change between PARTICLE and WAVE, demonstrating the principles of quantum mechanics.

### Perception Filter

```bash
cat perception.txt     # Shows current reality filters
```

- This file outlines the active filters that shape your perception of the filesystem. It can include various lenses through which you can view the world.

### Timeline Tracker

```bash
cat timeline.txt       # Tracks your journey through time
```

- This file records significant events and changes in your journey, helping you reflect on your past decisions and their impacts.

## Philosophical Domains

### 1. Logic Path (/logic)

- **Theme:** Truth and Paradox
- **Challenge:** Resolve self-referential paradoxes.
- **Keywords:** "paradox", "truth"

### 2. Emotion Path (/emotion)

- **Theme:** Feeling and Consciousness
- **Challenge:** Experience pure emotional states.
- **Keywords:** "joy", "curiosity", "doubt"

### 3. Identity Path (/identity)

- **Theme:** Self and Change
- **Challenge:** Understand persistence through change.
- **Keywords:** "change", "constant"

### 4. Time Path (/time)

- **Theme:** Temporal Experience
- **Challenge:** Grasp past and future simultaneously.
- **Keywords:** "present", "future"

### 5. Creation Path (/creation)

- **Theme:** Existence and Making
- **Challenge:** Create three unique elements.
- **Progress:** Tracked in created_elements.

### 6. History Path (/history)

- **Theme:** Reflection and Memory
- **Challenge:** Reflect on past decisions and their impacts.
- **Keywords:** "memory", "reflection"

### 7. Myth Path (/myth)

- **Theme:** Storytelling and Belief
- **Challenge:** Decode mythological stories to unlock new areas.
- **Keywords:** "myth", "story"

### 8. Perception Path (/perception)

- **Theme:** Reality and Illusion
- **Challenge:** Alter perception files to change how directories appear.
- **Keywords:** "reality", "illusion"

### 9. Quantum Path (/quantum)

- **Theme:** Uncertainty and Potential
- **Challenge:** Experience superposition and uncertainty in file states.
- **Keywords:** "quantum", "uncertainty"

### 10. Chaos Path (/chaos)

- **Theme:** Order and Disorder
- **Challenge:** Navigate an unpredictable environment.
- **Keywords:** "chaos", "order"

### 11. The Ultimate Question

- **Challenge:** Synthesize knowledge from all directories to answer the game's central philosophical question.
- **Outcome:** The answer shapes the filesystem's final state and your understanding.

## Progression System

1. Each response must be thoughtful (>50 characters).
2. Paths must be completed in sequence.
3. Progress is tracked in `progress.txt`.
4. Special files provide additional insights and mechanics.

## Advanced Interactions

### Quantum Observations

- Each read of `quantum_state.txt` causes wave function collapse.
- States alternate between PARTICLE and WAVE.
- Coherence values vary randomly.

### Reality Filters

Available filters:

- **Truth Lens:** Reveals hidden paths.
- **Quantum Vision:** Shows superposition states.
- **Temporal Sight:** Views timeline variations.

### Timeline Manipulation

- Past responses influence future options.
- Timeline stability affects available choices.
- Events are recorded chronologically.

## Tips for Deep Engagement

1. Consider multiple perspectives.
2. Reference philosophical concepts.
3. Make personal connections.
4. Question assumptions.
5. Explore paradoxes.
6. Observe quantum states.
7. Track timeline changes.

## Administrator Guide

### Setup

To run the Eternal Filesystem, use the following command:

```bash
cargo run --example eternal_fs --features demo -- ./eternal_root
```

### File Structure

```
eternal_root/
├── progress.txt
├── quantum_state.txt
├── perception.txt
├── timeline.txt
├── logic/
│   ├── README.txt
│   ├── question.txt
│   ├── answer.txt (user-created)
│   └── system_response.txt (auto-generated)
├── emotion/
│   └── [same structure]
└── [other philosophical domains...]
```

### Monitoring

- Check `progress.txt` for stage advancement.
- Monitor `quantum_state.txt` for state collapses.
- Review `timeline.txt` for temporal changes.
- Examine `perception.txt` for active filters.

### Troubleshooting

1. **Permission issues:** Check mount permissions.
2. **Missing responses:** Verify file write permissions.
3. **Stuck progression:** Ensure responses meet length and keyword requirements.
4. **Quantum state issues:** Verify random number generation.
5. **Timeline inconsistencies:** Check system time synchronization.
