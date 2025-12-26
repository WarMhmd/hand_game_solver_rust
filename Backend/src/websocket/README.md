# WebSocket Module

This module handles WebSocket connections and messaging for the game.

## Architecture

The WebSocket system uses an **mpsc channel-based architecture** with **acknowledgment tracking**:

1. When a WebSocket connection is established, an `UnboundedSender<Message>` is created
2. This sender is stored in the `Player` struct when they join a game
3. Messages can be sent to players through their sender, which forwards to their WebSocket connection
4. The system can wait for player acknowledgments before proceeding with game logic

## Module Structure

The module has been reorganized for better maintainability:

```
websocket/
├── mod.rs           # Module exports
├── handler.rs       # WebSocket connection handling
├── events.rs        # Event handlers and send/wait utilities
├── messages.rs      # Incoming message types (WsMessage)
├── responses.rs     # Outgoing response types (WsResponse)
└── README.md        # This file
```

### File Purposes

- **`handler.rs`** - Handles WebSocket upgrade, connection lifecycle, and message routing
- **`events.rs`** - Contains all event handlers and reusable send-and-wait patterns
- **`messages.rs`** - Defines all incoming message types from clients (`WsMessage` enum and related structs)
- **`responses.rs`** - Defines all outgoing message types to clients (`WsResponse` enum)
- **`mod.rs`** - Public API exports

## Message Types

### Incoming Messages (messages.rs)

```rust
WsMessage::Join { game_id, player_id }
WsMessage::StartGame { game_id, player_id }
WsMessage::RoundStarted { game_id, player_id, round_number }
WsMessage::DrawPhaseAck { game_id, player_id, round_number }
WsMessage::DrawPhaseFinished { game_id, player_id, round_number, data }
WsMessage::PlayingPhaseStarted { game_id, player_id, round_number, data }
WsMessage::Ping
```

### Outgoing Responses (responses.rs)

```rust
WsResponse::Joined { success, message }
WsResponse::GameStarted
WsResponse::RoundStarted { round_number, hand, fire_card_id, melded }
WsResponse::DrawPhase { player_id }
WsResponse::DrawnCard { player_id, card, is_fire_card }
WsResponse::PlayingPhaseStarted { player_id }
WsResponse::Error { message }
WsResponse::Pong
```

## Event Handler Pattern

The `events.rs` file uses a **generic send-and-wait pattern** to reduce code duplication:

### Generic Functions

1. **`send_and_wait<T>`** - Generic function that:
   - Creates channels for acknowledgments
   - Stores them in the tracker
   - Sends messages
   - Waits for responses with timeout
   - Cleans up after completion

2. **`handle_ack<T>`** - Generic function that:
   - Retrieves the session from tracker
   - Extracts the correct channel type
   - Sends the acknowledgment data
   - Handles type mismatches

### Event Categories

Events are organized into logical sections:

1. **Game Setup Events** - Join, start game
2. **Message Sending Utilities** - Send to player, broadcast
3. **Generic Pattern** - Reusable send-and-wait logic
4. **Round Started Events** - Round initialization
5. **Draw Phase Events** - Card drawing decisions
6. **Draw Card Events** - Card drawing completion
7. **Playing Phase Events** - Playing melds, discarding

## Usage Examples

### Sending a Message to a Specific Player

```rust
use crate::websocket::{send_to_player, WsResponse};

if let Some(sender) = &player.sender {
    let message = WsResponse::DrawPhase {
        player_id: player.id.clone(),
    };
    send_to_player(sender, message);
}
```

### Broadcasting to All Players

```rust
use crate::websocket::{broadcast_to_game, WsResponse};

let message = WsResponse::GameStarted;
broadcast_to_game(&game.players, message).await?;
```

### Waiting for Player Acknowledgment

```rust
use crate::websocket::send_round_start_and_wait;
use std::time::Duration;

// Prepare player data
let players_data = players.iter().map(|p| {
    (p.id.clone(), p.hand.clone(), p.fire_card_id.clone(), p.melded)
}).collect();

// Send and wait for all players to acknowledge
send_round_start_and_wait(
    &game_id,
    round_number,
    players_data,
    &players,
    state,
    Duration::from_secs(5),
).await?;
```

### Waiting for Player Data Response

```rust
use crate::websocket::send_player_draw_phase_and_wait;

// Send draw phase message and wait for player's choice
let draw_choice = send_player_draw_phase_and_wait(
    &active_player,
    &game_id,
    round_number,
    state,
).await?;

// Use the draw choice data
match draw_choice.draw_choice {
    DecideDrawResult::DrawFromDeck => { /* ... */ }
    DecideDrawResult::DrawFromDiscard => { /* ... */ }
}
```

## Acknowledgment System

The system tracks player acknowledgments using oneshot channels stored in `AppState.ack_trackers`.

### Session Keys

- `"{game_id}:round_{round_number}"` - Round start acknowledgments
- `"DrawPhase-{game_id}-{round_number}-{player_id}"` - Draw phase responses
- `"DrawCard-{game_id}-{round_number}-{player_id}"` - Draw card acknowledgments
- `"PlayingPhase-{game_id}-{round_number}-{player_id}"` - Playing phase responses

### AckSender Types

```rust
pub enum AckSender {
    Simple(oneshot::Sender<()>),           // For void acknowledgments
    Draw(oneshot::Sender<DrawPhaseData>),  // For draw choices
    Playing(oneshot::Sender<PlayingPhaseData>), // For playing actions
}
```

## Client-Side Integration

### Acknowledging Round Start

```typescript
socket.addEventListener('message', (event) => {
    const data = JSON.parse(event.data);
    
    if (data.event === 'roundStarted') {
        // Update UI
        updateGameState(data.hand, data.fire_card_id, data.melded);
        
        // Send acknowledgment
        socket.send(JSON.stringify({
            event: 'roundStarted',
            gameId: currentGameId,
            playerId: currentPlayerId,
            round_number: data.round_number
        }));
    }
});
```

### Responding with Data

```typescript
socket.addEventListener('message', (event) => {
    const data = JSON.parse(event.data);
    
    if (data.event === 'drawPhase') {
        // Show draw UI, get user choice
        const choice = await getUserDrawChoice();
        
        // Send response with data
        socket.send(JSON.stringify({
            event: 'drawPhaseFinished',
            gameId: currentGameId,
            playerId: currentPlayerId,
            round_number: currentRound,
            data: {
                draw_choice: choice // "DrawFromDeck" or "DrawFromDiscard"
            }
        }));
    }
});
```

## Adding New Events

1. **Add message type** in `messages.rs`:
   ```rust
   #[serde(rename_all = "camelCase")]
   NewEvent {
       game_id: String,
       player_id: String,
       // ... other fields
   }
   ```

2. **Add response type** in `responses.rs` (if needed):
   ```rust
   #[serde(rename_all = "camelCase")]
   NewResponse {
       // ... response fields
   }
   ```

3. **Create handler** in `events.rs`:
   ```rust
   pub async fn handle_new_event(...) -> Result<(), String> {
       // Implementation
   }
   ```

4. **Route in handler.rs**:
   ```rust
   WsMessage::NewEvent { ... } => {
       handle_new_event(...).await?;
       None
   }
   ```

## Benefits of This Organization

1. **Reduced Duplication**: Generic `send_and_wait` and `handle_ack` functions eliminate repetitive code
2. **Clear Separation**: Messages and responses are in separate files for clarity
3. **Maintainable**: Each file has a single, clear purpose
4. **Type Safety**: Strong typing for all message and response types
5. **Scalable**: Easy to add new events following the established patterns

## Error Handling

- **Timeouts**: All wait operations have configurable timeouts
- **Disconnections**: Failed sends are handled gracefully
- **Type Mismatches**: Acknowledgment type validation prevents incorrect channel usage
- **Session Cleanup**: Automatic cleanup prevents memory leaks

## Notes

- Messages are sent asynchronously through channels
- If a player disconnects, their sender will be dropped and messages will fail silently
- Always check if `player.sender.is_some()` before attempting to send
- The sender is set when a player joins the game via the `Join` event
- All acknowledgment sessions are automatically cleaned up after completion or timeout