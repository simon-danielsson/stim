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

	pub fn load_track(&mut self, track: Track) {
		self.sink.stop(); // stop previous track
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

	/// play/pause
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

	pub fn get_volume(&mut self) -> f32 {
		self.volume
	}
	pub fn set_volume(&mut self, vol: f32) {
		self.volume = vol.clamp(0.0, 2.0);
		self.sink.set_volume(self.volume);
	}

	pub fn current_track(&self) -> Option<Track> {
		self.current_track.clone()
	}

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
