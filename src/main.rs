use crossterm::{
	event::{self, Event, KeyCode},
	execute,
	terminal::{EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode, enable_raw_mode},
};
use ratatui::{
	Terminal,
	backend::CrosstermBackend,
	layout::{Constraint, Direction, Layout},
	style::{Color, Modifier, Style},
	text::{Line, Span, Text},
	widgets::{Block, BorderType, Borders, List, ListItem, ListState, Paragraph},
};
use std::io;

use crate::player::Player;

pub mod load_album_and_track_lists;
pub mod player;
const APP_VER: &str = env!("CARGO_PKG_VERSION");

fn main() -> std::io::Result<()> {
	let (track_list, album_list) = load_album_and_track_lists::run();

	// setup terminal
	enable_raw_mode()?;
	let mut stdout = io::stdout();
	execute!(stdout, EnterAlternateScreen)?;
	let backend = CrosstermBackend::new(stdout);
	let mut terminal = Terminal::new(backend)?;

	// init app state
	let mut app = App::new(album_list, track_list);

	// app
	loop {
		terminal.draw(|f| {
			let size = f.area();

			let vertical_chunks = Layout::default()
				.direction(Direction::Vertical)
				.constraints([
					Constraint::Max(1),    // info bar
					Constraint::Fill(1),   // main
					Constraint::Length(3), // player
				])
				.split(size);

			let horizontal_chunks = Layout::default()
				.direction(Direction::Horizontal)
				.constraints([
					Constraint::Percentage(50), // albums
					Constraint::Percentage(25), // tracks
					Constraint::Percentage(25), // queue
				])
				.split(vertical_chunks[1]);

			let infobar_style = Style::default();
			let player_style = Style::default();

			let highlight_style =
				Style::default().fg(Color::Red).add_modifier(Modifier::BOLD);

			// info bar
			let git_url = "https://github.com/simon-danielsson/stim";
			let git_span = Span::styled(
				format!("{}", git_url),
				Style::default()
					.fg(Color::Red)
					.add_modifier(Modifier::UNDERLINED),
			);

			let app_title_left = format!("stim v{}", APP_VER);
			let space = " "
				.repeat(size.width as usize - app_title_left.len() - git_url.len());
			let info_line =
				Line::from(vec![app_title_left.into(), space.into(), git_span]);

			let infobar = Paragraph::new(Text::from(vec![info_line.clone()]))
				.style(infobar_style);

			f.render_widget(infobar, vertical_chunks[0]);

			// albums
			let album_items: Vec<ListItem> = app
				.albums
				.iter()
				.map(|album| {
					ListItem::new(format!("{} - {}", album.artist, album.name))
				})
				.collect();

			let albums = List::new(album_items)
				.block(Block::default()
					.title("󰀥 Albums")
					.title_alignment(ratatui::layout::Alignment::Center)
					.borders(Borders::ALL)
					.border_type(BorderType::Rounded))
				.highlight_style(highlight_style)
				.highlight_symbol(">> ");

			f.render_stateful_widget(
				albums,
				horizontal_chunks[0],
				&mut app.album_state,
			);

			// tracks
			let track_items: Vec<ListItem> = app
				.tracks
				.iter()
				.map(|t| {
					ListItem::new(format!("{} - {}", t.track_num, t.track_name))
				})
				.collect();

			let tracks = List::new(track_items)
				.block(Block::default()
					.title(" Tracks")
					.title_alignment(ratatui::layout::Alignment::Center)
					.borders(Borders::ALL)
					.border_type(BorderType::Rounded))
				.highlight_style(highlight_style)
				.highlight_symbol(">> ");

			f.render_stateful_widget(
				tracks,
				horizontal_chunks[1],
				&mut app.track_state,
			);

			// queue
			let queue_items: Vec<ListItem> =
				app.queue
					.iter()
					.map(|t| {
						ListItem::new(format!(
							"{} - {}",
							t.artist, t.track_name
						))
					})
					.collect();

			let queue = List::new(queue_items)
				.block(Block::default()
					.title("󰲹 Queue")
					.title_alignment(ratatui::layout::Alignment::Center)
					.borders(Borders::ALL)
					.border_type(BorderType::Rounded))
				.highlight_style(highlight_style)
				.highlight_symbol(">> ");

			f.render_stateful_widget(queue, horizontal_chunks[2], &mut app.queue_state);

			// player
			let player_title = if let Some(track) = app.player.current_track() {
				format!(
					" {} - {} ({})",
					track.artist, track.track_name, track.album
				)
			} else {
				" No track".to_string()
			};

			let player = Paragraph::new(app.current_track_time())
				.style(player_style)
				.block(Block::default()
					.title(player_title)
					.title_alignment(ratatui::layout::Alignment::Center)
					.borders(Borders::ALL)
					.border_type(BorderType::Rounded));
			f.render_widget(player, vertical_chunks[2]);
		})?;

		// event handling
		let current_vol = app.player.get_volume();
		if event::poll(std::time::Duration::from_millis(200))? {
			if let Event::Key(key) = event::read()? {
				match key.code {
					// quit
					KeyCode::Char('q') => break,

					// player
					KeyCode::Char(' ') => {
						app.player.toggle_play();
					}
					KeyCode::Char('[') => {
						app.player.set_volume(current_vol - 0.1)
					}
					KeyCode::Char(']') => {
						app.player.set_volume(current_vol + 0.1)
					}
					// KeyCode::Char('{') => app.player.load_track(),
					KeyCode::Char('}') => {
						app.player.set_volume(current_vol + 0.1)
					}

					// panel nav
					KeyCode::Left | KeyCode::Char('n') | KeyCode::Char('h') => {
						app.move_left()
					}
					KeyCode::Right
					| KeyCode::Char('i')
					| KeyCode::Char('l') => app.move_right(),

					// list nav
					KeyCode::Down | KeyCode::Char('e') | KeyCode::Char('j') => {
						app.move_down()
					}
					KeyCode::Up | KeyCode::Char('o') | KeyCode::Char('k') => {
						app.move_up()
					}

					// clear queue
					KeyCode::Esc => app.clear_queue(),

					KeyCode::Char('a') => {
						app.main_action();
					}
					KeyCode::Char('A') => {
						app.aux_main_action();
					}

					_ => {}
				}
			}
			app.load_next_track_automatically();
		}
	}
	disable_raw_mode()?;
	execute!(terminal.backend_mut(), LeaveAlternateScreen)?;
	Ok(())
}

// application state

pub struct App {
	active_panel: ActivePanel,
	albums: Vec<load_album_and_track_lists::Album>,
	tracks: Vec<load_album_and_track_lists::Track>,
	queue: Vec<load_album_and_track_lists::Track>,

	album_state: ListState,
	track_state: ListState,
	queue_state: ListState,

	player: player::Player,
}

#[derive(Debug, Clone, Copy)]
enum ActivePanel {
	Albums,
	Tracks,
	Queue,
}

impl App {
	fn new(
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
		}
	}

	// navigation
	fn move_left(&mut self) {
		self.active_panel = match self.active_panel {
			ActivePanel::Albums => ActivePanel::Albums,
			ActivePanel::Tracks => ActivePanel::Albums,
			ActivePanel::Queue => ActivePanel::Tracks,
		}
	}

	fn move_right(&mut self) {
		self.active_panel = match self.active_panel {
			ActivePanel::Albums => ActivePanel::Tracks,
			ActivePanel::Tracks => ActivePanel::Queue,
			ActivePanel::Queue => ActivePanel::Queue,
		}
	}

	fn move_down(&mut self) {
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

	fn move_up(&mut self) {
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

	// queue logic
	fn main_action(&mut self) {
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

	fn aux_main_action(&mut self) {
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

	fn clear_queue(&mut self) {
		self.queue.clear();
		self.queue_state.select(None);
	}
	fn load_next_track_automatically(&mut self) {
		if self.player.sink.empty() {
			if !self.queue.is_empty() {
				let next_track = self.queue.remove(0);
				self.player.load_track(next_track);
				self.queue_state.select(Some(0));
			}
		}
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
