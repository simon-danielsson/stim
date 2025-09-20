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
	widgets::{Block, BorderType, Borders, Paragraph},
};
use std::io;

const APP_VER: &str = env!("CARGO_PKG_VERSION");

fn main() -> std::io::Result<()> {
	// Setup terminal
	enable_raw_mode()?;
	let mut stdout = io::stdout();
	execute!(stdout, EnterAlternateScreen)?;
	let backend = CrosstermBackend::new(stdout);
	let mut terminal = Terminal::new(backend)?;

	// Initialize app state
	let mut app = App::new();

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

			let albums_style = if matches!(app.active_panel, ActivePanel::Albums) {
				Style::default().fg(Color::Red).add_modifier(Modifier::BOLD)
			} else {
				Style::default()
			};
			let tracks_style = if matches!(app.active_panel, ActivePanel::Tracks) {
				Style::default().fg(Color::Red).add_modifier(Modifier::BOLD)
			} else {
				Style::default()
			};
			let queue_style = if matches!(app.active_panel, ActivePanel::Queue) {
				Style::default().fg(Color::Red).add_modifier(Modifier::BOLD)
			} else {
				Style::default()
			};

			let git_url = "https://github.com/simon-danielsson";
			let git_span = Span::styled(
				format!("{}", git_url),
				Style::default()
					.fg(Color::Blue)
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

			let albums =
				Paragraph::new("Albums")
					.style(albums_style)
					.block(Block::default()
						.title("󰀥 Albums")
						.title_alignment(ratatui::layout::Alignment::Center)
						.borders(Borders::ALL)
						.border_type(BorderType::Rounded));
			f.render_widget(albums, horizontal_chunks[0]);

			let tracks =
				Paragraph::new("Tracks")
					.style(tracks_style)
					.block(Block::default()
						.title(" Tracks")
						.title_alignment(ratatui::layout::Alignment::Center)
						.borders(Borders::ALL)
						.border_type(BorderType::Rounded));
			f.render_widget(tracks, horizontal_chunks[1]);

			let queue =
				Paragraph::new("Queue")
					.style(queue_style)
					.block(Block::default()
						.title("󰲹 Queue")
						.title_alignment(ratatui::layout::Alignment::Center)
						.borders(Borders::ALL)
						.border_type(BorderType::Rounded));
			f.render_widget(queue, horizontal_chunks[2]);

			let player = Paragraph::new("timeline, duration/total duration")
				.style(player_style)
				.block(Block::default()
					.title(" artist - song (album)")
					.title_alignment(ratatui::layout::Alignment::Center)
					.borders(Borders::ALL)
					.border_type(BorderType::Rounded));
			f.render_widget(player, vertical_chunks[2]);
		})?;
		if event::poll(std::time::Duration::from_millis(200))? {
			if let Event::Key(key) = event::read()? {
				match key.code {
					// General
					KeyCode::Char('q') => break, // quit
					// KeyCode::Char('s') => sort(),
					// KeyCode::Char('f') => find(),
					// KeyCode::Char('c') => clear_find(),

					// Move left
					KeyCode::Char('n') => app.move_left(),
					KeyCode::Char('h') => app.move_left(),
					KeyCode::Left => app.move_left(),
					// Move down
					// KeyCode::Char('e') => app.move_down(),
					// KeyCode::Char('j') => app.move_down(),
					// KeyCode::Down => app.move_down(),
					// Move up
					// KeyCode::Char('o') => app.move_up(),
					// KeyCode::Char('k') => app.move_up(),
					// KeyCode::Up => app.move_up(),
					// Move right
					KeyCode::Char('i') => app.move_right(),
					KeyCode::Char('l') => app.move_right(),
					KeyCode::Right => app.move_right(),
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
}

#[derive(Debug, Clone, Copy)]
enum ActivePanel {
	Albums,
	Tracks,
	Queue,
}

impl App {
	fn new() -> Self {
		Self {
			active_panel: ActivePanel::Albums,
		}
	}
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
}
