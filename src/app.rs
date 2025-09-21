use crate::load_album_and_track_lists;
use crate::player;
use crate::player::Player;

use ratatui::{layout::Rect, widgets::ListState};

pub struct App {
	pub active_panel: ActivePanel,
	pub albums: Vec<load_album_and_track_lists::Album>,
	pub tracks: Vec<load_album_and_track_lists::Track>,
	pub queue: Vec<load_album_and_track_lists::Track>,

	pub album_state: ListState,
	pub track_state: ListState,
	pub queue_state: ListState,

	pub player: player::Player,

	pub input: String,
	pub input_mode: InputMode,
	pub find_term: String,
	pub find_char_index: usize,
}

#[derive(Debug, Clone, Copy)]
pub enum ActivePanel {
	Albums,
	Tracks,
	Queue,
}

#[derive(Debug, Clone, Copy)]
pub enum InputMode {
	Normal,
	Find,
}

impl App {
	pub fn new(
		albums: Vec<load_album_and_track_lists::Album>,
		tracks: Vec<load_album_and_track_lists::Track>,
	) -> Self {
		let mut album_state = ListState::default();
		album_state.select(Some(0));

		let mut track_state = ListState::default();
		track_state.select(Some(0));

		let mut queue_state = ListState::default();
		queue_state.select(Some(0));

		Self {
			active_panel: ActivePanel::Albums,
			albums,
			tracks,
			queue: Vec::new(),
			album_state,
			track_state,
			queue_state,
			player: Player::new(),
			input: String::new(),
			find_term: String::new(),
			input_mode: InputMode::Normal,
			find_char_index: 0,
		}
	}

	// navigation
	pub fn move_left(&mut self) {
		self.active_panel = match self.active_panel {
			ActivePanel::Albums => ActivePanel::Albums,
			ActivePanel::Tracks => ActivePanel::Albums,
			ActivePanel::Queue => ActivePanel::Tracks,
		}
	}

	pub fn move_right(&mut self) {
		self.active_panel = match self.active_panel {
			ActivePanel::Albums => ActivePanel::Tracks,
			ActivePanel::Tracks => ActivePanel::Queue,
			ActivePanel::Queue => ActivePanel::Queue,
		}
	}

	pub fn move_down(&mut self) {
		match self.active_panel {
			ActivePanel::Albums => {
				let i = match self.album_state.selected() {
					Some(i) if i < self.albums.len() - 1 => i + 1,
					Some(i) => i,
					None => 0,
				};
				self.album_state.select(Some(i));
			}
			ActivePanel::Tracks => {
				let i = match self.track_state.selected() {
					Some(i) if i < self.tracks.len() - 1 => i + 1,
					Some(i) => i,
					None => 0,
				};
				self.track_state.select(Some(i));
			}
			ActivePanel::Queue => {
				let i = match self.queue_state.selected() {
					Some(i) if i < self.queue.len().saturating_sub(1) => i + 1,
					Some(i) => i,
					None => 0,
				};
				self.queue_state.select(Some(i));
			}
		}
	}

	pub fn move_up(&mut self) {
		match self.active_panel {
			ActivePanel::Albums => {
				let i = match self.album_state.selected() {
					Some(i) if i > 0 => i - 1,
					Some(i) => i,
					None => 0,
				};
				self.album_state.select(Some(i));
			}
			ActivePanel::Tracks => {
				let i = match self.track_state.selected() {
					Some(i) if i > 0 => i - 1,
					Some(i) => i,
					None => 0,
				};
				self.track_state.select(Some(i));
			}
			ActivePanel::Queue => {
				let i = match self.queue_state.selected() {
					Some(i) if i > 0 => i - 1,
					Some(i) => i,
					None => 0,
				};
				self.queue_state.select(Some(i));
			}
		}
	}

	pub fn main_action(&mut self) {
		match self.active_panel {
			ActivePanel::Albums => {
				if let Some(i) = self.album_state.selected() {
					let mut tracks = self.albums[i].tracks.clone();
					self.queue.extend(tracks.drain(..));
					self.queue_state
						.select(Some(self.queue.len().saturating_sub(1)));
				}
				if let Some(first_track) = self.queue.first() {
					self.player.load_track(first_track.clone());
				}
				if self.player.current_track().is_none() {
					if let Some(first_track) = self.queue.first() {
						self.player.load_track(first_track.clone());
					}
				}
			}
			ActivePanel::Tracks => {
				if let Some(i) = self.track_state.selected() {
					self.queue.push(self.tracks[i].clone());
					self.queue_state
						.select(Some(self.queue.len().saturating_sub(1)));
				}
				if let Some(first_track) = self.queue.first() {
					self.player.load_track(first_track.clone());
				}
				if self.player.current_track().is_none() {
					if let Some(first_track) = self.queue.first() {
						self.player.load_track(first_track.clone());
					}
				}
			}
			ActivePanel::Queue => {
				if let Some(i) = self.queue_state.selected() {
					self.queue.remove(i);
					if self.queue.is_empty() {
						self.queue_state.select(None);
					} else if i >= self.queue.len() {
						self.queue_state.select(Some(self.queue.len() - 1));
					}
				}
			}
		}
	}

	pub fn aux_main_action(&mut self) {
		match self.active_panel {
			ActivePanel::Albums => {
				if let Some(i) = self.album_state.selected() {
					let mut tracks = self.albums[i].tracks.clone();
					while let Some(t) = tracks.pop() {
						self.queue.insert(0, t);
					}
					self.queue_state.select(Some(0));
				}
			}
			ActivePanel::Tracks => {
				if let Some(i) = self.track_state.selected() {
					let t = self.tracks[i].clone();
					self.queue.insert(0, t);
					self.queue_state.select(Some(0));
				}
			}
			ActivePanel::Queue => {
				if let Some(i) = self.queue_state.selected() {
					if i < self.queue.len() {
						let t = self.queue.remove(i);
						self.queue.insert(0, t);
						self.queue_state.select(Some(0));
					}
				}
			}
		}
	}

	pub fn clear_queue(&mut self) {
		self.queue.clear();
		self.queue_state.select(None);
	}

	// find
	pub fn move_cursor_left(&mut self) {
		let cursor_moved_left = self.find_char_index.saturating_sub(1);
		self.find_char_index = self.clamp_cursor(cursor_moved_left);
	}

	pub fn move_cursor_right(&mut self) {
		let cursor_moved_right = self.find_char_index.saturating_add(1);
		self.find_char_index = self.clamp_cursor(cursor_moved_right);
	}

	pub fn enter_char(&mut self, new_char: char) {
		let index = self.byte_index();
		self.input.insert(index, new_char);
		self.move_cursor_right();
	}
	fn byte_index(&self) -> usize {
		self.input
			.char_indices()
			.map(|(i, _)| i)
			.nth(self.find_char_index)
			.unwrap_or(self.input.len())
	}
	pub fn delete_char(&mut self) {
		let is_not_cursor_leftmost = self.find_char_index != 0;
		if is_not_cursor_leftmost {
			// Method "remove" is not used on the saved text for deleting the selected char.
			// Reason: Using remove on String works on bytes instead of the chars.
			// Using remove would require special care because of char boundaries.

			let current_index = self.find_char_index;
			let from_left_to_current_index = current_index - 1;

			// Getting all characters before the selected character.
			let before_char_to_delete =
				self.input.chars().take(from_left_to_current_index);
			// Getting all characters after selected character.
			let after_char_to_delete = self.input.chars().skip(current_index);

			// Put all characters together except the selected one.
			// By leaving the selected one out, it is forgotten and therefore deleted.
			self.input = before_char_to_delete.chain(after_char_to_delete).collect();
			self.move_cursor_left();
		}
	}

	fn clamp_cursor(&self, new_cursor_pos: usize) -> usize {
		new_cursor_pos.clamp(0, self.input.chars().count())
	}

	fn reset_cursor(&mut self) {
		self.find_char_index = 0;
	}

	pub fn submit_find(&mut self) {
		self.find_term = self.input.clone();
		self.input_mode = InputMode::Normal;
		self.reset_cursor();
	}

	pub fn clear_find(&mut self) {
		self.input.clear();
	}

	// player
	pub fn load_next_track_automatically(&mut self) {
		if self.player.sink.empty() {
			if !self.queue.is_empty() {
				let next_track = self.queue.remove(0);
				self.player.load_track(next_track);
				self.queue_state.select(Some(0));
			}
		}
	}

	pub fn update_player_timeline(&self, player_chunk: Rect) -> String {
		if let Some(track) = &self.player.current_track {
			let elapsed = self.player.position().as_secs() as usize;
			let total = track.length as usize;
			let width = player_chunk.width as usize;
			let progress = if total > 0 {
				(elapsed * width) / total
			} else {
				0
			};
			let bar = format!(
				"{}{}",
				"█".repeat(progress),
				"░".repeat(width.saturating_sub(progress))
			);
			return bar;
		}
		String::new()
	}

	pub fn current_track_time(&self) -> String {
		if let Some(track) = &self.player.current_track {
			let elapsed = self.player.position().as_secs();
			let total = &track.length;
			format!(
				"{:02}:{:02} / {:02}:{:02}",
				elapsed / 60,
				elapsed % 60,
				total / 60,
				total % 60
			)
		} else {
			"00:00 / 00:00".to_string()
		}
	}
}
