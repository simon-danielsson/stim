use crate::load_album_and_track_lists::Track;
use rodio::{Decoder, OutputStreamBuilder, Sink};
use std::{
	fs::File,
	io::BufReader,
	time::{Duration, Instant},
};

pub struct Player {
	pub sink: Sink,
	_stream: rodio::OutputStream, // keep alive
	pub current_track: Option<Track>,
	last_start_time: Option<Instant>,
	current_pos: Duration,
	volume: f32,
}

impl Player {
	/// Create a new player, initially empty (no track loaded)
	pub fn new() -> Self {
		let stream = OutputStreamBuilder::open_default_stream().unwrap();
		let mixer = stream.mixer();
		let sink = Sink::connect_new(mixer);

		Self {
			sink,
			_stream: stream,
			current_track: None,
			last_start_time: None,
			current_pos: Duration::ZERO,
			volume: 0.75,
		}
	}

	/// Load a track into the player and start playback
	pub fn load_track(&mut self, track: Track) {
		self.sink.stop(); // stop any previous track
		let mixer = self._stream.mixer();
		self.sink = Sink::connect_new(mixer);

		let file = File::open(&track.path).unwrap();
		let source = Decoder::new(BufReader::new(file)).unwrap();
		self.sink.append(source);

		self.current_track = Some(track);
		self.current_pos = Duration::ZERO;
		self.last_start_time = Some(Instant::now());
		self.sink.set_volume(self.volume);
		self.sink.play();
	}

	/// Toggle play/pause
	pub fn toggle_play(&mut self) {
		if self.sink.is_paused() {
			self.sink.play();
			self.last_start_time = Some(Instant::now());
		} else {
			if let Some(start) = self.last_start_time.take() {
				self.current_pos += start.elapsed();
			}
			self.sink.pause();
		}
	}

	/// Set the volume (0.0–2.0)
	pub fn get_volume(&mut self) -> f32 {
		self.volume
	}
	pub fn set_volume(&mut self, vol: f32) {
		self.volume = vol.clamp(0.0, 2.0);
		self.sink.set_volume(self.volume);
	}

	/// Get the currently loaded track
	pub fn current_track(&self) -> Option<Track> {
		self.current_track.clone()
	}

	/// Get the current playback position
	pub fn position(&self) -> Duration {
		if self.sink.is_paused() {
			self.current_pos
		} else if let Some(start) = self.last_start_time {
			self.current_pos + start.elapsed()
		} else {
			self.current_pos
		}
	}
}

/// Example run function interacting with a queue
pub fn run(track: &Track, input: &str, previous_tracks: &mut Vec<Track>) -> Player {
	let mut player = Player::new();

	match input.trim() {
		"p" => {
			if let Some(_) = player.current_track() {
				player.toggle_play();
			} else {
				player.load_track(track.clone());
			}
		}
		"]" => player.set_volume(player.volume + 0.1),
		"[" => player.set_volume(player.volume - 0.1),
		"{" => {
			if let Some(prev) = previous_tracks.last() {
				player.load_track(prev.clone());
			}
		}
		"}" => {
			player.load_track(track.clone());
			previous_tracks.push(track.clone());
		}
		_ => {}
	}

	player
}
