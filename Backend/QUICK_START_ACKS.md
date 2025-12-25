# Quick Start: Player Acknowledgment System

This guide shows you how to wait for all players to acknowledge receiving a message before proceeding.

## TL;DR - The Easy Way

Use `send_round_start_and_wait()` in your game logic:

```rust
use crate::websocket::send_round_start_and_wait;
use std::time::Duration;

// In your start_game function where you have AppState
pub async fn start_game_async(&mut self, state: &Arc<AppState>) -> Result<(), String> {
    for r in 1..=self.max_rounds {
        let mut round_state = init_round(self);
        
        // Prepare player data
        let players_data: Vec<_> = round_state.players.iter().map(|p| {
            (
                p.id.clone(),
                p.hand.clone(),
                p.fire_card_id.clone(),
                p.melded,
            )
        }).collect();
        
        // Convert ActivePlayer to Player for the function
        let player_refs: Vec<Player> = round_state.players.iter().map(|ap| Player {
            id: ap.id.clone(),
            name: ap.name.clone(),
            bot_strategy: ap.bot_strategy.clone(),
            score: ap.score,
            did_join: ap.did_join,
            sender: ap.sender.clone(),
        }).collect();

        // Send round start and wait for all players (5 second timeout)
        match send_round_start_and_wait(
            &self.id,
            r,
            players_data,
            &player_refs,
            state,
            Duration::from_secs(5),
        ).await {
            Ok(()) => {
                println!("✅ All players ready for round {}", r);
                // Continue with game logic
            }
            Err(e) => {
                println!("⚠️ Timeout or error: {}", e);
                return Err(e);
            }
        }
        
        // Rest of your round logic...
    }
    Ok(())
}
```

## Client-Side (JavaScript/TypeScript)

Your frontend MUST send back an acknowledgment:

```typescript
socket.onmessage = (event) => {
    const data = JSON.parse(event.data);
    
    if (data.event === 'roundStarted') {
        // 1. Update your UI first
        updateGameState({
            roundNumber: data.round_number,
            hand: data.hand,
            fireCardId: data.fire_card_id,
            melded: data.melded
        });
        
        // 2. Send acknowledgment back
        socket.send(JSON.stringify({
            event: 'roundStarted',
            gameId: currentGameId,
            playerId: currentPlayerId,
            round_number: data.round_number
        }));
    }
};
```

## How It Works

1. **Server**: Calls `send_round_start_and_wait()` which:
   - Sends personalized `RoundStarted` message to each player
   - Creates acknowledgment trackers for each player
   - Waits for all players to respond (with timeout)

2. **Client**: Receives message and sends back:
   ```json
   {
     "event": "roundStarted",
     "gameId": "game-123",
     "playerId": "player-456",
     "round_number": 1
   }
   ```

3. **Server**: Once all acknowledgments received, continues execution

## Important Notes

- ✅ Always use a timeout (3-10 seconds recommended)
- ✅ Handle errors gracefully (decide if you abort or continue)
- ✅ Clients MUST send acknowledgment or server will timeout
- ✅ The function is async, so you need an async context

## Alternative Functions

If you need more control:

### `broadcast_and_wait_for_acks()`
Sends the same message to all players (not personalized):

```rust
use crate::websocket::{broadcast_and_wait_for_acks, WsResponse};

let message = WsResponse::RoundStarted {
    round_number: 1,
    hand: vec![],
    fire_card_id: None,
    melded: false,
};

broadcast_and_wait_for_acks(
    &game_id,
    round_number,
    &players,
    message,
    &state,
    Duration::from_secs(5),
).await?;
```

### `handle_round_started_ack()`
Manually handle individual acknowledgments (advanced usage only)

## Testing Tips

1. Test with slow network by delaying client acknowledgment
2. Test timeout by not sending acknowledgment from one client
3. Monitor logs for "✅ Player X acknowledged round Y" messages

## Troubleshooting

**Problem**: Server times out waiting for acknowledgments
- **Solution**: Check client is sending acknowledgment message
- **Solution**: Increase timeout duration
- **Solution**: Check WebSocket connection is stable

**Problem**: "Session not found" error
- **Solution**: Client is acknowledging too late (after timeout)
- **Solution**: Client sent wrong game_id or round_number

**Problem**: Game logic is blocking
- **Solution**: Make sure your game logic function is `async`
- **Solution**: Call it from an async context (e.g., in a tokio::spawn)

## Example: Converting Existing Code

### Before (No waiting):
```rust
pub fn start_game(&mut self) {
    for r in 1..=self.max_rounds {
        // Send messages
        for player in &round_state.players {
            if let Some(sender) = &player.sender {
                // Send message...
            }
        }
        // Immediately continue (players might not have received message yet)
        run_round_logic();
    }
}
```

### After (With waiting):
```rust
pub async fn start_game_async(&mut self, state: &Arc<AppState>) -> Result<(), String> {
    for r in 1..=self.max_rounds {
        let round_state = init_round(self);
        
        // Prepare data
        let players_data = /* ... */;
        let player_refs = /* ... */;
        
        // Wait for all players to acknowledge
        send_round_start_and_wait(
            &self.id, r, players_data, &player_refs, state, Duration::from_secs(5)
        ).await?;
        
        // Now safe to continue - all players have the data
        run_round_logic();
    }
    Ok(())
}
```

## Need More Help?

- See `Backend/ACKNOWLEDGMENT_EXAMPLE.md` for detailed examples
- See `Backend/src/websocket/README.md` for architecture details
- Check `Backend/src/websocket/events.rs` for implementation