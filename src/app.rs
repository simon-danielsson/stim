use crate::load_album_and_track_lists;
use crate::player;
use crate::player::Player;
use rand::rng;
use rand::seq::SliceRandom; // provides shuffle
use ratatui::style::Color;
use ratatui::{
	layout::Rect,
	widgets::{ListState, TableState},
};
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;
use std::usize;

pub struct App {
	pub active_panel: ActivePanel,
	pub albums: Vec<load_album_and_track_lists::Album>,
	pub tracks: Vec<load_album_and_track_lists::Track>,
	pub all_albums: Vec<load_album_and_track_lists::Album>,
	pub all_tracks: Vec<load_album_and_track_lists::Track>,
	pub queue: Vec<load_album_and_track_lists::Track>,

	pub album_state: TableState,
	pub track_state: TableState,
	pub queue_state: ListState,
	pub queue_index: Option<usize>,

	pub sort_state: SortState,

	pub player: player::Player,

	pub input: String,
	pub input_mode: InputMode,
	pub find_term: String,
	pub find_char_index: usize,

	pub highlight_color: Color,
}

pub enum SortState {
	AZ,
	ZA,
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

#[derive(Serialize, Deserialize)]
pub struct AppConfig {
	highlight_color: u8,
}
impl AppConfig {
	pub fn get_color(&self) -> Color {
		match self.highlight_color {
			0 => Color::Red,
			1 => Color::LightRed,
			2 => Color::Blue,
			3 => Color::LightBlue,
			4 => Color::Cyan,
			5 => Color::LightCyan,
			6 => Color::Magenta,
			7 => Color::LightMagenta,
			8 => Color::Yellow,
			9 => Color::Green,
			10 => Color::LightGreen,
			_ => Color::Blue,
		}
	}

	pub fn set_color(&mut self, color: Color) {
		self.highlight_color = match color {
			Color::Red => 0,
			Color::LightRed => 1,
			Color::Blue => 2,
			Color::LightBlue => 3,
			Color::Cyan => 4,
			Color::LightCyan => 5,
			Color::Magenta => 6,
			Color::LightMagenta => 7,
			Color::Yellow => 8,
			Color::Green => 9,
			Color::LightGreen => 10,
			_ => 2, // default to Blue
		}
	}
	pub fn load(path: &PathBuf) -> Self {
		if let Ok(contents) = fs::read_to_string(path) {
			serde_json::from_str(&contents).unwrap_or_default()
		} else {
			Self::default()
		}
	}

	/// Save AppConfig to file
	pub fn save(&self, path: &PathBuf) {
		if let Ok(json) = serde_json::to_string_pretty(self) {
			let _ = fs::write(path, json);
		}
	}
}

impl Default for AppConfig {
	fn default() -> Self {
		Self {
			highlight_color: 0, // default to Blue
		}
	}
}

impl App {
	pub fn new(
		albums: Vec<load_album_and_track_lists::Album>,
		tracks: Vec<load_album_and_track_lists::Track>,
		highlight_color: Color,
	) -> Self {
		let mut album_state = TableState::default();
		album_state.select(Some(0));

		let mut track_state = TableState::default();
		track_state.select(Some(0));

		let mut queue_state = ListState::default();
		queue_state.select(Some(0));

		Self {
			active_panel: ActivePanel::Albums,
			all_albums: albums.clone(),
			all_tracks: tracks.clone(),
			albums,
			tracks,
			queue: Vec::new(),
			album_state,
			track_state,
			queue_state,
			sort_state: SortState::AZ,
			queue_index: Some(0),
			player: Player::new(),
			input: String::new(),
			find_term: String::new(),
			input_mode: InputMode::Normal,
			find_char_index: 0,
			highlight_color,
		}
	}

	// highlight color
	pub fn rotate_hl_color(&mut self) {
		self.highlight_color = match self.highlight_color {
			Color::Red => Color::LightRed,
			Color::LightRed => Color::Blue,
			Color::Blue => Color::LightBlue,
			Color::LightBlue => Color::Cyan,
			Color::Cyan => Color::LightCyan,
			Color::LightCyan => Color::Magenta,
			Color::Magenta => Color::LightMagenta,
			Color::LightMagenta => Color::Yellow,
			Color::Yellow => Color::Green,
			Color::Green => Color::LightGreen,
			Color::LightGreen => Color::Red,
			_ => Color::Red,
		};
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
				if self.player.current_track().is_none() {
					self.start_play_at(0);
				}
			}
			ActivePanel::Tracks => {
				if let Some(i) = self.track_state.selected() {
					self.queue.push(self.tracks[i].clone());
					self.queue_state
						.select(Some(self.queue.len().saturating_sub(1)));
				}
				if self.player.current_track().is_none() {
					self.start_play_at(0);
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

	// queue
	pub fn add_all_tracks_to_queue(&mut self) {
		for i in self.tracks.clone() {
			self.queue.push(i)
		}
		self.queue_state
			.select(Some(self.queue.len().saturating_sub(1)));
		if self.player.current_track().is_none() {
			self.start_play_at(0);
		}
	}

	pub fn clear_queue(&mut self) {
		self.queue.clear();
		self.queue_state.select(None);
		self.queue_index = None;
	}
	pub fn shuffle_queue(&mut self) {
		let mut rng = rng();
		self.queue.shuffle(&mut rng);

		if !self.queue.is_empty() {
			self.queue_state.select(Some(0));
			self.queue_index = Some(0);
		} else {
			self.queue_state.select(None);
			self.queue_index = None;
		}
	}

	// sort

	pub fn toggle_sort(&mut self) {
		self.sort_state = match self.sort_state {
			SortState::AZ => SortState::ZA,
			SortState::ZA => SortState::AZ,
		};
		self.sort_lists();
	}

	pub fn sort_lists(&mut self) {
		match self.sort_state {
			SortState::AZ => {
				self.albums.sort_by(|a, b| {
					a.artist.to_lowercase().cmp(&b.artist.to_lowercase())
				});
				self.tracks.sort_by(|a, b| {
					a.track_name
						.to_lowercase()
						.cmp(&b.track_name.to_lowercase())
				});
			}
			SortState::ZA => {
				self.albums.sort_by(|a, b| {
					b.artist.to_lowercase().cmp(&a.artist.to_lowercase())
				});
				self.tracks.sort_by(|a, b| {
					b.track_name
						.to_lowercase()
						.cmp(&a.track_name.to_lowercase())
				});
			}
		}
		self.album_state.select(Some(0));
		self.track_state.select(Some(0));
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
			let queue_index = self.find_char_index;
			let from_left_to_queue_index = queue_index - 1;
			let before_char_to_delete =
				self.input.chars().take(from_left_to_queue_index);
			let after_char_to_delete = self.input.chars().skip(queue_index);
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

	pub fn find_albums(&mut self) {
		if self.find_term.is_empty() {
			self.albums = self.all_albums.clone();
		} else {
			let term = self.find_term.to_lowercase();
			self.albums = self
				.all_albums
				.iter()
				.filter(|album| {
					album.name.to_lowercase().contains(&term)
						|| album.artist.to_lowercase().contains(&term)
				})
				.cloned()
				.collect();
		}
		self.album_state.select(Some(0));
	}

	pub fn find_tracks(&mut self) {
		if self.find_term.is_empty() {
			self.tracks = self.all_tracks.clone();
		} else {
			let term = self.find_term.to_lowercase();
			self.tracks = self
				.all_tracks
				.iter()
				.filter(|track| {
					track.track_name.to_lowercase().contains(&term)
						|| track.artist.to_lowercase().contains(&term)
				})
				.cloned()
				.collect();
		}
		self.track_state.select(Some(0));
	}

	pub fn clear_find(&mut self) {
		self.input.clear();
		self.find_term.clear();
		self.albums = self.all_albums.clone();
		self.tracks = self.all_tracks.clone();
		self.album_state.select(Some(0));
		self.track_state.select(Some(0));
	}

	// player

	fn start_play_at(&mut self, index: usize) {
		if index >= self.queue.len() {
			return;
		}
		self.queue_index = Some(index);
		self.queue_state.select(Some(index));
		let track = self.queue[index].clone();
		self.player.load_track(track);
	}

	pub fn load_next_track_if_current_ends(&mut self) {
		if self.player.sink.empty() {
			if self.queue.is_empty() {
				self.queue_index = None;
				self.queue_state.select(None);
				return;
			}
			match self.queue_index {
				Some(i) if i + 1 < self.queue.len() => {
					self.start_play_at(i + 1);
				}
				Some(_) => {
					self.player.sink.pause();
					self.queue_index = None;
					self.queue_state.select(None);
				}
				None => {
					// nothing playing but queue present -> start first
					self.start_play_at(0);
				}
			}
		}
	}

	pub fn next_track(&mut self) {
		if self.queue.is_empty() {
			self.player.sink.pause();
			self.queue_index = None;
			self.queue_state.select(None);
			return;
		}
		match self.queue_index {
			Some(i) if i + 1 < self.queue.len() => {
				self.start_play_at(i + 1);
			}
			Some(_) => {
				// at end of queue
				self.player.sink.pause();
				self.queue_index = None;
				self.queue_state.select(None);
			}
			None => {
				// nothing playing -> start first
				self.start_play_at(0);
			}
		}
	}

	pub fn prev_track(&mut self) {
		if self.queue.is_empty() {
			return;
		}
		match self.queue_index {
			Some(i) if i > 0 => {
				self.start_play_at(i - 1);
			}
			Some(_) => {
				// already at first track -> restart first track (or no-op)
				self.start_play_at(0);
			}
			None => {
				// not playing -> start first
				self.start_play_at(0);
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
