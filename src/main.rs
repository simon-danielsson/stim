use crossterm::{
	event::{self, Event, KeyCode, KeyEventKind},
	execute,
	terminal::{EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode, enable_raw_mode},
};
use ratatui::{
	Terminal,
	backend::CrosstermBackend,
	layout::{Alignment, Constraint, Layout, Position},
	style::{Color, Modifier, Style},
	text::Text,
	widgets::{Block, BorderType, Borders, List, ListItem, Paragraph},
};

pub mod app;
pub mod constants;
pub mod load_album_and_track_lists;
pub mod player;

use crate::app::*;
use crate::constants::*;

fn main() -> std::io::Result<()> {
	enable_raw_mode()?;
	let mut stdout = std::io::stdout();
	execute!(stdout, EnterAlternateScreen)?;
	let backend = CrosstermBackend::new(stdout);
	let mut terminal = Terminal::new(backend)?;

	let (track_list, album_list) = load_album_and_track_lists::run();

	let mut config_path = dirs::config_dir().unwrap_or_else(|| ".".into());
	config_path.push("my_app");
	std::fs::create_dir_all(&config_path).ok();
	config_path.push("app_config.json");

	let app_config = AppConfig::load(&config_path);

	// init app state
	let mut app = App::new(album_list, track_list, app_config.get_color());
	app.sort_lists();

	// app
	loop {
		terminal.draw(|f| {
			let hl_color = app.highlight_color;
			let size = f.area();
			let vertical_chunks = Layout::vertical([
				Constraint::Length(4), // player
				Constraint::Fill(1),   // main
			])
			.split(size);
			let horizontal_chunks = Layout::horizontal([
				Constraint::Percentage(40), // albums
				Constraint::Percentage(40), // tracks
				Constraint::Fill(1),        // queue
			])
			.split(vertical_chunks[1]);
			let queue_logo_chunk = Layout::vertical([
				Constraint::Max(3),
				Constraint::Fill(1),
				Constraint::Max(8),
			])
			.split(horizontal_chunks[2]);

			let highlight_style = Style::default()
				.fg(Color::Black)
				.add_modifier(Modifier::BOLD)
				.bg(hl_color);

			// albums
			let album_has_focus = matches!(app.active_panel, ActivePanel::Albums);

			let album_items: Vec<ListItem> = app
				.albums
				.iter()
				.map(|album| {
					ListItem::new(format!("{} - {}", album.artist, album.name))
				})
				.collect();
			let albums = List::new(album_items.clone())
				.block({
					let mut block = Block::default()
						.title("󰀥 Albums")
						.title_alignment(ratatui::layout::Alignment::Center)
						.borders(Borders::ALL)
						.border_type(BorderType::Rounded);
					if album_has_focus {
						block = block.border_style(
							Style::default().fg(hl_color),
						);
					}
					block
				})
				.highlight_style(if album_has_focus {
					highlight_style
				} else {
					Style::default()
				})
				.highlight_symbol(if album_has_focus { "  " } else { "   " });

			f.render_stateful_widget(
				albums,
				horizontal_chunks[0],
				&mut app.album_state,
			);

			// tracks
			let tracks_has_focus = matches!(app.active_panel, ActivePanel::Tracks);
			let track_items: Vec<ListItem> = app
				.tracks
				.iter()
				.map(|track| {
					ListItem::new(format!(
						"{} - {} [{}]",
						track.artist, track.track_name, track.album
					))
				})
				.collect();
			let tracks = List::new(track_items.clone())
				.block({
					let mut block = Block::default()
						.title(" Tracks")
						.title_alignment(ratatui::layout::Alignment::Center)
						.borders(Borders::ALL)
						.border_type(BorderType::Rounded);
					if tracks_has_focus {
						block = block.border_style(
							Style::default().fg(hl_color),
						);
					}
					block
				})
				.highlight_style(if tracks_has_focus {
					highlight_style
				} else {
					Style::default()
				})
				.highlight_symbol(if tracks_has_focus { "  " } else { "   " });
			f.render_stateful_widget(
				tracks,
				horizontal_chunks[1],
				&mut app.track_state,
			);

			// find
			let find = Paragraph::new(app.input.as_str())
				.style(match app.input_mode {
					InputMode::Normal => Style::default(),
					InputMode::Find => Style::default().fg(hl_color),
				})
				.block(Block::default()
					.title(" Find")
					.title_alignment(ratatui::layout::Alignment::Center)
					.borders(Borders::ALL)
					.border_type(BorderType::Rounded));
			f.render_widget(find, queue_logo_chunk[0]);

			// queue
			let queue_has_focus = matches!(app.active_panel, ActivePanel::Queue);
			let queue_items: Vec<ListItem> = app
				.queue
				.iter()
				.enumerate()
				.map(|(i, track)| {
					let mut item = ListItem::new(format!(
						"{}. {} - {} [{}]",
						track.track_num,
						track.artist,
						track.track_name,
						track.album
					));
					if Some(i) == app.queue_index {
						item = item.style(Style::default()
							.add_modifier(Modifier::BOLD)
							.bg(Color::DarkGray));
					}
					item
				})
				.collect();
			let queue = List::new(queue_items)
				.block({
					let mut block = Block::default()
						.title("󰲹 Queue")
						.title_alignment(ratatui::layout::Alignment::Center)
						.borders(Borders::ALL)
						.border_type(BorderType::Rounded);
					if queue_has_focus {
						block = block.border_style(
							Style::default().fg(hl_color),
						);
					}
					block
				})
				.highlight_style(if queue_has_focus {
					highlight_style
				} else {
					Style::default()
				})
				.highlight_symbol(if queue_has_focus { "  " } else { "   " });
			f.render_stateful_widget(queue, queue_logo_chunk[1], &mut app.queue_state);

			// logo
			let logo_text = format!(
				"\n░█▀▀░▀█▀░▀█▀░█▄█\n░▀▀█░░█░░░█░░█░█\n░▀▀▀░░▀░░▀▀▀░▀░▀\nv{}\n{}\n{}\n",
				APP_VER, WEBSITE, COPYRIGHT
			);
			let centered_lines: String = logo_text
				.lines()
				.map(|line| {
					let total_padding = (queue_logo_chunk[1].width as usize)
						.saturating_sub(line.chars().count());
					let left_padding = total_padding / 2;
					let right_padding = total_padding - left_padding;
					format!(
						"{}{}{}\n",
						" ".repeat(left_padding), // ░
						line,
						" ".repeat(right_padding)
					)
				})
				.collect();
			let logo = Paragraph::new(centered_lines)
				.style(Style::default().fg(hl_color))
				.alignment(Alignment::Left);
			f.render_widget(logo, queue_logo_chunk[2]);

			// player
			let player_timeline_str = app.update_player_timeline(vertical_chunks[0]);
			let total_width = vertical_chunks[0].width as usize;
			let right = format!("󰕾 {}%", app.player.get_volume_as_percentage());
			let max_left = total_width.saturating_sub(right.chars().count() + 2);
			let mut left_full = format!(
				"{} │ {}",
				app.current_track_time(),
				if let Some(track) = app.player.current_track() {
					format!(
						"{}. {} - {} [{}]",
						track.track_num,
						track.artist,
						track.track_name,
						track.album
					)
				} else {
					"No track".to_string()
				}
			);
			if left_full.chars().count() > max_left {
				left_full = left_full
					.chars()
					.take(max_left.saturating_sub(1))
					.collect();
				left_full.push('…');
			}
			let used = left_full.chars().count() + right.chars().count();
			let padding = if total_width > used {
				total_width - used - 2
			} else {
				1
			};
			let player_ui_text = format!(
				"{}{}{}  \n{}",
				left_full,
				" ".repeat(padding),
				right,
				player_timeline_str
			);
			let player_ui = Paragraph::new(Text::raw(player_ui_text))
				.style(Style::default().fg(app.highlight_color))
				.block(Block::default()
					.borders(Borders::ALL)
					.border_type(BorderType::Rounded)
					.border_style(Style::default().fg(Color::White)));
			f.render_widget(player_ui, vertical_chunks[0]);

			// draw cursor in find field
			match app.input_mode {
				InputMode::Normal => {}
				#[allow(clippy::cast_possible_truncation)]
				InputMode::Find => f.set_cursor_position(Position::new(
					queue_logo_chunk[0].x + app.find_char_index as u16 + 1,
					queue_logo_chunk[0].y + 1,
				)),
			}
			app.load_next_track_if_current_ends();
		})?;

		// event handling
		let current_vol = app.player.get_volume();
		if event::poll(std::time::Duration::from_millis(100))? {
			if let Event::Key(key) = event::read()? {
				match app.input_mode {
					InputMode::Normal => match key.code {
						c if K_LEFT.contains(&c) => app.move_left(),
						c if K_RIGHT.contains(&c) => app.move_right(),
						c if K_DOWN.contains(&c) => app.move_down(),
						c if K_UP.contains(&c) => app.move_up(),

						K_QUIT => break,
						K_FIND => {
							if !app.input.is_empty() {
								app.input = "".to_string();
							}
							app.input_mode = InputMode::Find;
						}
						K_CLEAR_FIND => app.clear_find(),
						K_PLAY => app.player.toggle_play(),

						K_SORT => app.toggle_sort(),

						K_N_TRK => app.next_track(),
						K_P_TRK => app.prev_track(),

						K_VOL_DOWN => {
							app.player.set_volume(current_vol - 0.1)
						}
						K_VOL_UP => {
							app.player.set_volume(current_vol + 0.1)
						}
						K_HL => app.rotate_hl_color(),

						// queue
						K_CLEAR => app.clear_queue(),
						K_SHUFFLE => app.shuffle_queue(),
						K_ADD_ALL_TRACKS => app.add_all_tracks_to_queue(),
						K_MAIN => app.main_action(),
						K_AUX => app.aux_main_action(),
						_ => {}
					},

					InputMode::Find if key.kind == KeyEventKind::Press => {
						match key.code {
							KeyCode::Enter => app.submit_find(),
							KeyCode::Char(to_insert) => {
								app.enter_char(to_insert);
								app.find_term = app.input.clone();
								app.find_albums();
								app.find_tracks();
							}
							KeyCode::Backspace => {
								app.delete_char();
								app.find_albums();
								app.find_tracks();
							}
							KeyCode::Left => app.move_cursor_left(),
							KeyCode::Right => app.move_cursor_right(),
							KeyCode::Esc => {
								app.input_mode = InputMode::Normal;
							}
							_ => {}
						}
					}
					InputMode::Find => {}
				}
			}
		}
	}
	let mut config = AppConfig::default();
	config.set_color(app.highlight_color);
	config.save(&config_path);

	std::mem::drop(app);
	disable_raw_mode()?;
	execute!(terminal.backend_mut(), LeaveAlternateScreen)?;
	Ok(())
}
