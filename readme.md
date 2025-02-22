# ChessRL - Terminal-based Chess with Reinforcement Learning

A terminal-based chess game featuring a pure reinforcement learning engine that learns and improves during gameplay.


## Features

- Play chess directly in your terminal
- Clean TUI interface with move history and analytics
- Real-time engine analysis and thinking process display
- Pure reinforcement learning engine that improves as you play
- No pre-trained models - watch the engine learn from scratch each game

## Installation

```bash
curl -sSL https://raw.githubusercontent.com/yourusername/chessrl/main/install.sh | bash
```

## Usage

Start a new game:
```bash
chessrl
```

### Making Moves

Moves are made using algebraic coordinates. Format: `<from square> <to square>`

Examples:
- Move pawn from a2 to a4: `a2 a4`
- Move knight from b1 to c3: `b1 c3`

### Controls

- Scroll: View move history
- q: Quit game
- ESC: return to main menu (game is not saved)

## Engine

The reinforcement learning engine:
- Learns purely through self-play during each game
- Does not persist learned knowledge between games (yet)
- Displays its thinking process and move confidence in real-time
- Improves noticeably as the game progresses

## Coming Soon

- Persistent learning (saved models)
- Opening book integration
- Additional analysis features
- Network play
- recommended moves

## License

MIT
