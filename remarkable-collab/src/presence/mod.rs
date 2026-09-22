//! Presence tracking for collaborative editing.
//!
//! Tracks user cursors, selection state, and online status.

use std::collections::HashMap;
use std::time::{Duration, Instant};

use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Cursor position on a page.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct CursorPosition {
    pub x: f32,
    pub y: f32,
    pub page_id: Uuid,
}

/// User presence state.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserPresence {
    /// User's unique ID.
    pub user_id: Uuid,
    /// Display name.
    pub display_name: String,
    /// User's color (for cursor/selection display).
    pub color: UserColor,
    /// Current cursor position.
    pub cursor: Option<CursorPosition>,
    /// Currently selected items.
    pub selection: Vec<Uuid>,
    /// Current active tool.
    pub active_tool: Option<String>,
    /// Whether user is actively drawing.
    pub is_drawing: bool,
    /// Last activity timestamp.
    #[serde(skip)]
    pub last_seen: Option<Instant>,
}

impl UserPresence {
    pub fn new(user_id: Uuid, display_name: String) -> Self {
        Self {
            user_id,
            display_name,
            color: UserColor::random(),
            cursor: None,
            selection: Vec::new(),
            active_tool: None,
            is_drawing: false,
            last_seen: Some(Instant::now()),
        }
    }

    /// Update last seen time.
    pub fn touch(&mut self) {
        self.last_seen = Some(Instant::now());
    }

    /// Check if user is considered online (seen within timeout).
    pub fn is_online(&self, timeout: Duration) -> bool {
        self.last_seen
            .map(|t| t.elapsed() < timeout)
            .unwrap_or(false)
    }
}

/// Colors for user presence display.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct UserColor {
    pub r: u8,
    pub g: u8,
    pub b: u8,
}

impl UserColor {
    pub const fn new(r: u8, g: u8, b: u8) -> Self {
        Self { r, g, b }
    }

    /// Generate a random user color from a predefined palette.
    pub fn random() -> Self {
        use std::time::SystemTime;
        let seed = SystemTime::now()
            .duration_since(SystemTime::UNIX_EPOCH)
            .unwrap()
            .as_nanos() as usize;
        
        Self::PALETTE[seed % Self::PALETTE.len()]
    }

    /// Predefined color palette for users.
    pub const PALETTE: [UserColor; 10] = [
        UserColor::new(66, 133, 244),   // Blue
        UserColor::new(234, 67, 53),    // Red
        UserColor::new(251, 188, 5),    // Yellow
        UserColor::new(52, 168, 83),    // Green
        UserColor::new(156, 39, 176),   // Purple
        UserColor::new(255, 87, 34),    // Orange
        UserColor::new(0, 188, 212),    // Cyan
        UserColor::new(233, 30, 99),    // Pink
        UserColor::new(63, 81, 181),    // Indigo
        UserColor::new(121, 85, 72),    // Brown
    ];

    /// Convert to CSS color string.
    pub fn to_css(&self) -> String {
        format!("rgb({}, {}, {})", self.r, self.g, self.b)
    }

    /// Convert to hex color string.
    pub fn to_hex(&self) -> String {
        format!("#{:02x}{:02x}{:02x}", self.r, self.g, self.b)
    }
}

/// Manages presence for all users in a session.
#[derive(Debug, Default)]
pub struct PresenceTracker {
    /// User presence by user ID.
    users: HashMap<Uuid, UserPresence>,
    /// Timeout for considering a user offline.
    timeout: Duration,
}

impl PresenceTracker {
    pub fn new(timeout: Duration) -> Self {
        Self {
            users: HashMap::new(),
            timeout,
        }
    }

    /// Add or update a user.
    pub fn upsert(&mut self, presence: UserPresence) {
        self.users.insert(presence.user_id, presence);
    }

    /// Update cursor position for a user.
    pub fn update_cursor(&mut self, user_id: Uuid, cursor: CursorPosition) {
        if let Some(presence) = self.users.get_mut(&user_id) {
            presence.cursor = Some(cursor);
            presence.touch();
        }
    }

    /// Update selection for a user.
    pub fn update_selection(&mut self, user_id: Uuid, selection: Vec<Uuid>) {
        if let Some(presence) = self.users.get_mut(&user_id) {
            presence.selection = selection;
            presence.touch();
        }
    }

    /// Set drawing state for a user.
    pub fn set_drawing(&mut self, user_id: Uuid, is_drawing: bool) {
        if let Some(presence) = self.users.get_mut(&user_id) {
            presence.is_drawing = is_drawing;
            presence.touch();
        }
    }

    /// Remove a user.
    pub fn remove(&mut self, user_id: &Uuid) -> Option<UserPresence> {
        self.users.remove(user_id)
    }

    /// Get a user's presence.
    pub fn get(&self, user_id: &Uuid) -> Option<&UserPresence> {
        self.users.get(user_id)
    }

    /// Get all online users.
    pub fn online_users(&self) -> Vec<&UserPresence> {
        self.users
            .values()
            .filter(|p| p.is_online(self.timeout))
            .collect()
    }

    /// Get all users (including offline).
    pub fn all_users(&self) -> Vec<&UserPresence> {
        self.users.values().collect()
    }

    /// Clean up stale users (offline beyond timeout).
    pub fn cleanup_stale(&mut self, max_offline: Duration) {
        self.users.retain(|_, presence| {
            presence
                .last_seen
                .map(|t| t.elapsed() < max_offline)
                .unwrap_or(false)
        });
    }

    /// Touch a user to update their last seen time.
    pub fn touch(&mut self, user_id: &Uuid) {
        if let Some(presence) = self.users.get_mut(user_id) {
            presence.touch();
        }
    }

    /// Get presence updates since checking (for broadcasting).
    pub fn get_presence_snapshot(&self) -> Vec<UserPresence> {
        self.users.values().cloned().collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_user_presence() {
        let mut presence = UserPresence::new(Uuid::new_v4(), "Alice".to_string());
        assert!(presence.is_online(Duration::from_secs(5)));

        presence.cursor = Some(CursorPosition {
            x: 100.0,
            y: 200.0,
            page_id: Uuid::new_v4(),
        });
        assert!(presence.cursor.is_some());
    }

    #[test]
    fn test_presence_tracker() {
        let mut tracker = PresenceTracker::new(Duration::from_secs(30));
        
        let user1 = UserPresence::new(Uuid::new_v4(), "Alice".to_string());
        let user2 = UserPresence::new(Uuid::new_v4(), "Bob".to_string());
        
        tracker.upsert(user1.clone());
        tracker.upsert(user2.clone());
        
        assert_eq!(tracker.online_users().len(), 2);
        
        tracker.remove(&user1.user_id);
        assert_eq!(tracker.online_users().len(), 1);
    }

    #[test]
    fn test_user_color() {
        let color = UserColor::new(255, 128, 64);
        assert_eq!(color.to_css(), "rgb(255, 128, 64)");
        assert_eq!(color.to_hex(), "#ff8040");
    }
}
