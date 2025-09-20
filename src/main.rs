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

pub mod load_album_and_track_lists;

const APP_VER: &str = env!("CARGO_PKG_VERSION");

fn main() -> std::io::Result<()> {
	let (track_list, album_list) = load_album_and_track_lists::run();

	// Setup terminal
	enable_raw_mode()?;
	let mut stdout = io::stdout();
	execute!(stdout, EnterAlternateScreen)?;
	let backend = CrosstermBackend::new(stdout);
	let mut terminal = Terminal::new(backend)?;

	// Initialize app state
	let mut app = App::new(album_list, track_list);

	// App loop
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

			// Highlighting style
			let highlight_style =
				Style::default().fg(Color::Red).add_modifier(Modifier::BOLD);

			// ---------------- Info bar ----------------
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

			// ---------------- Albums ----------------
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

			// ---------------- Tracks ----------------
			let track_items: Vec<ListItem> = app
				.visible_tracks()
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

			// ---------------- Queue ----------------
			let queue =
				Paragraph::new("Queue (not implemented)").block(Block::default()
					.title("󰲹 Queue")
					.title_alignment(ratatui::layout::Alignment::Center)
					.borders(Borders::ALL)
					.border_type(BorderType::Rounded));

			f.render_widget(queue, horizontal_chunks[2]);

			// ---------------- Player ----------------
			let player = Paragraph::new("timeline, duration/total duration")
				.style(player_style)
				.block(Block::default()
					.title(" artist - song (album)")
					.title_alignment(ratatui::layout::Alignment::Center)
					.borders(Borders::ALL)
					.border_type(BorderType::Rounded));
			f.render_widget(player, vertical_chunks[2]);
		})?;

		// ---------------- Event handling ----------------
		if event::poll(std::time::Duration::from_millis(200))? {
			if let Event::Key(key) = event::read()? {
				match key.code {
					// Quit
					KeyCode::Char('q') => break,

					// Navigation between panels
					KeyCode::Left | KeyCode::Char('n') => app.move_left(),
					KeyCode::Right | KeyCode::Char('i') => app.move_right(),

					// Navigation inside lists
					KeyCode::Down | KeyCode::Char('e') => app.move_down(),
					KeyCode::Up | KeyCode::Char('o') => app.move_up(),

					// Select album to show tracks
					KeyCode::Enter => app.select_album(),

					_ => {}
				}
			}
		}
	}

	disable_raw_mode()?;
	execute!(terminal.backend_mut(), LeaveAlternateScreen)?;
	Ok(())
}

// ----------------------------
// Application state
// ----------------------------

struct App {
	active_panel: ActivePanel,
	albums: Vec<load_album_and_track_lists::Album>,
	tracks: Vec<load_album_and_track_lists::Track>,
	album_state: ListState,
	track_state: ListState,
	selected_album: Option<usize>,
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

		Self {
			active_panel: ActivePanel::Albums,
			albums,
			tracks,
			album_state,
			track_state,
			selected_album: None,
		}
	}

	// ---------------- Navigation ----------------
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
				let tracks = self.visible_tracks();
				let i = match self.track_state.selected() {
					Some(i) if i < tracks.len().saturating_sub(1) => i + 1,
					Some(i) => i,
					None => 0,
				};
				self.track_state.select(Some(i));
			}
			ActivePanel::Queue => {}
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
			ActivePanel::Queue => {}
		}
	}

	// ---------------- Selection ----------------
	fn select_album(&mut self) {
		if let Some(i) = self.album_state.selected() {
			self.selected_album = Some(i);
			self.track_state.select(Some(0));
		}
	}

	// ---------------- Visible tracks ----------------
	fn visible_tracks(&self) -> Vec<load_album_and_track_lists::Track> {
		if let Some(album_idx) = self.selected_album {
			self.albums[album_idx].tracks.clone()
		} else {
			self.tracks.clone() // all tracks if no album selected
		}
	}
}
