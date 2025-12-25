# WebSocket Module

This module handles WebSocket connections and messaging for the game.

## Architecture

The WebSocket system uses an **mpsc channel-based architecture**:

1. When a WebSocket connection is established, an `UnboundedSender<Message>` is created
2. This sender is stored in the `Player` struct when they join a game
3. Messages can be sent to players through their sender, which forwards to their WebSocket connection

## Components

- **`handler.rs`** - WebSocket connection handling and message routing
- **`events.rs`** - Event handlers and utility functions for messaging
- **`mod.rs`** - Module exports

## Usage Examples

### Sending a Message to a Specific Player

```rust
use crate::websocket::send_to_player;
use serde::Serialize;

#[derive(Serialize)]
struct GameUpdate {
    event: String,
    data: String,
}

// Assuming you have access to the player's sender
if let Some(sender) = &player.sender {
    let message = GameUpdate {
        event: "gameUpdate".to_string(),
        data: "Your turn!".to_string(),
    };
    send_to_player(sender, message);
}
```

### Broadcasting to All Players in a Game

```rust
use crate::websocket::broadcast_to_game;
use serde::Serialize;

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct GameStarted {
    event: String,
    game_id: String,
}

let message = GameStarted {
    event: "gameStarted".to_string(),
    game_id: game_id.clone(),
};

broadcast_to_game(&game_id, message, &state).await?;
```

### Finding a Player and Sending a Message

```rust
use crate::websocket::send_to_player;

let games = state.games.read().await;
if let Some(game) = games.get(&game_id) {
    if let Some(player) = game.players.iter().find(|p| p.id == player_id) {
        if let Some(sender) = &player.sender {
            send_to_player(sender, YourMessage { /* ... */ });
        }
    }
}
```

## Message Flow

1. **Client → Server**: Client sends JSON message over WebSocket
2. **Server Parses**: Message is parsed into `WsMessage` enum
3. **Event Handler**: Appropriate event handler processes the message
4. **Response**: Server can respond immediately or send messages later via the stored sender

## Key Types

- `UnboundedSender<Message>` - Channel sender for sending messages to a WebSocket connection
- `WsMessage` - Incoming message types from clients
- `WsResponse` - Outgoing response types to clients

## Adding New Events

1. Add a new variant to `WsMessage` in `handler.rs`
2. Add a new variant to `WsResponse` if needed
3. Create a handler function in `events.rs`
4. Add the handler to the match statement in `handle_ws_event()`

## Notes

- Messages are sent asynchronously through channels
- If a player disconnects, their sender will be dropped and messages will fail silently
- Always check if `player.sender.is_some()` before attempting to send
- The sender is set when a player joins the game via the `Join` event

## Waiting for Player Acknowledgments

The system supports waiting for all players to acknowledge receiving a message before proceeding.

### How It Works

1. **Broadcast with acknowledgment tracking**: Use `broadcast_and_wait_for_acks()` to send a message and wait
2. **Clients acknowledge**: Clients send back a `RoundStarted` event with their player_id
3. **Server proceeds**: Once all players acknowledge, the server continues execution

### Example: Waiting for Round Start Acknowledgments

```rust
use crate::websocket::{broadcast_and_wait_for_acks, WsResponse};
use std::time::Duration;

// In your game logic (e.g., start_game function)
pub async fn start_round(
    game_id: &str,
    round_number: i32,
    players: &Vec<Player>,
    state: &Arc<AppState>,
) -> Result<(), String> {
    // Prepare the round data for each player
    let mut messages_for_players = Vec::new();
    
    for player in players {
        let message = WsResponse::RoundStarted {
            round_number,
            hand: player.hand.clone(),
            fire_card_id: player.fire_card_id.clone(),
            melded: player.melded,
        };
        messages_for_players.push((player.id.clone(), message));
    }

    // Broadcast and wait for all players to acknowledge (with 5 second timeout)
    match broadcast_and_wait_for_acks(
        game_id,
        round_number,
        players,
        WsResponse::RoundStarted { /* ... */ },
        state,
        Duration::from_secs(5),
    ).await {
        Ok(()) => {
            println!("✅ All players acknowledged round start");
            // Continue with game logic...
        }
        Err(e) => {
            println!("⚠️ Failed to get all acknowledgments: {}", e);
            // Handle timeout or error...
        }
    }

    Ok(())
}
```

### Client-Side Example (JavaScript/TypeScript)

```typescript
// When receiving round started message
socket.addEventListener('message', (event) => {
    const data = JSON.parse(event.data);
    
    if (data.event === 'roundStarted') {
        const { round_number, hand, fire_card_id, melded } = data;
        
        // Update your UI with the round data
        updateGameState(hand, fire_card_id, melded);
        
        // Send acknowledgment back to server
        socket.send(JSON.stringify({
            event: 'roundStarted',
            gameId: currentGameId,
            playerId: currentPlayerId,
            round_number: round_number
        }));
    }
});
```

### Key Points

- **Timeout**: Always specify a reasonable timeout to avoid waiting forever
- **Error Handling**: Handle both timeout errors and acknowledgment failures
- **Session Keys**: The system tracks acknowledgments using `game_id:round_N` format
- **Automatic Cleanup**: Sessions are automatically cleaned up after completion or timeout
- **Player Filtering**: Only players with active senders are tracked for acknowledgments