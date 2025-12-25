# Acknowledgment System Example

This document shows how to use the acknowledgment system to wait for all players to confirm they've received a message before proceeding with game logic.

## The Problem

Previously, when broadcasting a "round started" message, the server would immediately continue without knowing if players received it:

```rust
// ❌ OLD WAY - No waiting
for player in &round_state.players {
    if let Some(sender) = &player.sender {
        let message = WsResponse::RoundStarted {
            round_number: r,
            hand: player.hand.clone(),
            fire_card_id: None,
            melded: false,
        };
        if let Ok(json) = serde_json::to_string(&message) {
            let _ = sender.send(Message::Text(json.clone().into()));
        }
    }
}
// Server continues immediately without knowing if players received the message
```

## The Solution

Use `broadcast_and_wait_for_acks()` to wait for all players to acknowledge:

```rust
// ✅ NEW WAY - Wait for acknowledgments
use crate::websocket::broadcast_and_wait_for_acks;
use std::time::Duration;

// In your GameState implementation
impl GameState {
    pub async fn start_game_async(&mut self, state: &Arc<AppState>) -> Result<(), String> {
        println!("=== HAND GAME SIMULATION START ===");

        for r in 1..=self.max_rounds {
            let mut round_state = init_round(self);
            
            println!("\n--- ROUND {} ---", r);
            
            // Create the message to broadcast
            let message = WsResponse::RoundStarted {
                round_number: r,
                hand: vec![], // This will be customized per player
                fire_card_id: None,
                melded: false,
            };

            // Broadcast and wait for all players to acknowledge (5 second timeout)
            match broadcast_and_wait_for_acks(
                &self.id,
                r,
                &round_state.players.iter().map(|ap| Player {
                    id: ap.id.clone(),
                    name: ap.name.clone(),
                    bot_strategy: ap.bot_strategy.clone(),
                    score: ap.score,
                    did_join: ap.did_join,
                    sender: ap.sender.clone(),
                }).collect(),
                message,
                state,
                Duration::from_secs(5),
            ).await {
                Ok(()) => {
                    println!("✅ All players ready for round {}", r);
                }
                Err(e) => {
                    println!("⚠️ Not all players acknowledged round {}: {}", r, e);
                    // You can choose to continue anyway or return an error
                    return Err(format!("Failed to start round {}: {}", r, e));
                }
            }

            // Now you can safely continue with the round logic
            // knowing all players have received and acknowledged the round start
            
            // ... rest of game logic ...
        }

        Ok(())
    }
}
```

## Better Approach: Send Individual Messages with Shared Wait

Since each player needs different data (their own hand), here's a better pattern:

```rust
use crate::websocket::{send_to_player, broadcast_and_wait_for_acks, WsResponse};
use std::time::Duration;

pub async fn start_round_with_acks(
    game_id: &str,
    round_number: i32,
    players: &Vec<ActivePlayer>,
    state: &Arc<AppState>,
) -> Result<(), String> {
    // Convert ActivePlayer to Player for the acknowledgment tracker
    let player_refs: Vec<Player> = players.iter().map(|ap| Player {
        id: ap.id.clone(),
        name: ap.name.clone(),
        bot_strategy: ap.bot_strategy.clone(),
        score: ap.score,
        did_join: ap.did_join,
        sender: ap.sender.clone(),
    }).collect();

    // Create a dummy message just for setting up the acknowledgment tracker
    // (we'll send individual messages below)
    let dummy_message = WsResponse::RoundStarted {
        round_number,
        hand: vec![],
        fire_card_id: None,
        melded: false,
    };

    // Set up acknowledgment tracking and get receivers
    let receivers = {
        let mut receivers = Vec::new();
        let mut ack_senders = std::collections::HashMap::new();

        for player in &player_refs {
            if player.sender.is_some() {
                let (tx, rx) = tokio::sync::oneshot::channel();
                ack_senders.insert(player.id.clone(), tx);
                receivers.push(rx);
            }
        }

        let session_key = format!("{}:round_{}", game_id, round_number);
        {
            let mut trackers = state.ack_trackers.write().await;
            trackers.insert(session_key.clone(), ack_senders);
        }
        
        receivers
    };

    // Send personalized messages to each player
    for player in players {
        if let Some(sender) = &player.sender {
            let message = WsResponse::RoundStarted {
                round_number,
                hand: player.hand.clone(),
                fire_card_id: player.fire_card_id.clone(),
                melded: player.melded,
            };
            send_to_player(sender, message);
        }
    }

    // Wait for all acknowledgments with timeout
    let session_key = format!("{}:round_{}", game_id, round_number);
    let wait_result = tokio::time::timeout(Duration::from_secs(5), async {
        for rx in receivers {
            if rx.await.is_err() {
                return Err("Player acknowledgment failed".to_string());
            }
        }
        Ok(())
    }).await;

    // Clean up the session
    {
        let mut trackers = state.ack_trackers.write().await;
        trackers.remove(&session_key);
    }

    match wait_result {
        Ok(Ok(())) => {
            println!("✅ All players acknowledged round {}", round_number);
            Ok(())
        }
        Ok(Err(e)) => Err(e),
        Err(_) => Err("Timeout waiting for all players to acknowledge".to_string()),
    }
}
```

## Client-Side Implementation

Your frontend clients must send back an acknowledgment:

```typescript
// JavaScript/TypeScript example
const socket = new WebSocket('ws://localhost:3000/ws');

socket.onmessage = (event) => {
    const data = JSON.parse(event.data);
    
    if (data.event === 'roundStarted') {
        const { round_number, hand, fire_card_id, melded } = data;
        
        // 1. Update your UI with the new round data
        updateGameUI({
            roundNumber: round_number,
            hand: hand,
            fireCardId: fire_card_id,
            melded: melded
        });
        
        // 2. IMPORTANT: Send acknowledgment back to server
        socket.send(JSON.stringify({
            event: 'roundStarted',
            gameId: currentGameId,
            playerId: currentPlayerId,
            round_number: round_number
        }));
        
        console.log(`✅ Acknowledged round ${round_number}`);
    }
};
```

## Integration with Current Code

To update your existing `start_game()` function:

```rust
// In logic.rs

// Change from synchronous to async
impl GameState {
    // Keep the old function for backwards compatibility
    pub fn start_game(&mut self) {
        // ... existing sync code ...
    }

    // Add new async version that waits for acknowledgments
    pub async fn start_game_with_acks(&mut self, state: &Arc<AppState>) -> Result<(), String> {
        let mut results: Vec<BotResult> = self
            .players
            .iter()
            .map(|p| BotResult {
                name: p.name.to_string(),
                total_score: 0,
                rounds_won: 0,
            })
            .collect();

        println!("=== HAND GAME SIMULATION START ===");

        for r in 1..=self.max_rounds {
            let mut round_state = init_round(self);
            
            println!("\n--- ROUND {} ---", r);

            // Wait for all players to acknowledge round start
            start_round_with_acks(
                &self.id,
                r,
                &round_state.players,
                state,
            ).await?;

            // Continue with round logic...
            // ... rest of your game logic ...
        }

        Ok(())
    }
}
```

## Key Points

1. **Timeout**: Always use a reasonable timeout (3-10 seconds) to avoid waiting forever
2. **Error Handling**: Decide if you want to abort on timeout or continue anyway
3. **Session Cleanup**: The system automatically cleans up acknowledgment sessions
4. **Client Response**: Clients MUST send back the acknowledgment message
5. **Async Context**: You need an async context (e.g., in an async handler or spawned task)

## Testing

You can test the acknowledgment system by:

1. Starting a game with multiple players
2. Having some players delay their acknowledgment
3. Observing the server waits for all players
4. Testing timeout scenarios by not sending acknowledgments

```rust
// Example test scenario
#[tokio::test]
async fn test_round_start_acks() {
    let state = Arc::new(AppState {
        games: Arc::new(RwLock::new(HashMap::new())),
        ack_trackers: Arc::new(RwLock::new(HashMap::new())),
    });

    // Create game with players...
    
    // Start round in background
    let handle = tokio::spawn(async move {
        start_round_with_acks("game1", 1, &players, &state).await
    });

    // Simulate client acknowledgments...
    
    // Should complete successfully
    assert!(handle.await.unwrap().is_ok());
}
```
