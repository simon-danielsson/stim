use lofty::file::TaggedFileExt;
use lofty::prelude::ItemKey;
use lofty::probe::Probe;
use std::collections::HashMap;
use std::env;
use std::fs;
use std::path::Path;
use walkdir::WalkDir;

pub fn run() -> (Vec<Track>, Vec<Album>) {
	let track_list: Vec<Track> = match create_song_list() {
		Ok(vector) => vector,
		Err(e) => panic!("Error occured when parsing songs: {}", e),
	};
	let album_list: Vec<Album> = match create_album_list(track_list.clone()) {
		Ok(vector) => vector,
		Err(e) => panic!("Error occured when compiling the list of albums: {}", e),
	};
	(track_list, album_list)
}

fn create_album_list(tracks_vec: Vec<Track>) -> std::io::Result<Vec<Album>> {
	let mut album_map: HashMap<(String, String), Vec<Track>> = HashMap::new();

	// group tracks by (artist, album)
	for track in tracks_vec {
		let key = (track.artist.clone(), track.album.clone());
		album_map.entry(key).or_insert(Vec::new()).push(track);
	}

	// convert into Vec<Album>
	let mut album_list: Vec<Album> = Vec::new();
	for ((artist, album_name), tracks) in album_map {
		// sort tracks by track_num
		let mut sorted_tracks = tracks;
		sorted_tracks.sort_by_key(|t| t.track_num);
		album_list.push(Album {
			artist,
			name: album_name,
			tracks: sorted_tracks,
		});
	}
	Ok(album_list)
}

#[derive(Debug)]
#[allow(dead_code)]
pub struct Album {
	pub artist: String,
	pub name: String,
	pub tracks: Vec<Track>,
}

#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct Track {
	pub artist: String,
	pub track_name: String,
	pub track_num: i32,
	pub path: String,
	pub album: String,
	pub in_queue: bool,
	pub currently_playing: bool,
}

impl Track {
	fn new(
		artist: String,
		track_name: String,
		track_num: i32,
		path: String,
		album: String,
	) -> Self {
		Self {
			artist,
			track_name,
			track_num,
			path,
			album,
			in_queue: false,
			currently_playing: false,
		}
	}
}

fn create_song_list() -> std::io::Result<Vec<Track>> {
	let stim_library_dir_path: String = match env::var("HOME") {
		Ok(home) => format!("{}/stim-library/", home),
		Err(e) => panic!("Home directory could not be found: {}", e),
	};
	// create stim library in home directory if it doesn't exist
	fs::create_dir_all(&stim_library_dir_path)?;
	let mut tracks_vec: Vec<Track> = Vec::new();
	let extensions_to_search_for = ["wav", "mp3", "flac"];
	for entry in WalkDir::new(&stim_library_dir_path)
		.into_iter()
		.filter_map(Result::ok)
		.filter(|e| e.file_type().is_file())
	{
		if let Some(ext) = entry.path().extension().and_then(|s| s.to_str()) {
			if extensions_to_search_for.contains(&ext) {
				let track: Track =
					match get_track_metadata(entry.path().to_str().unwrap()) {
						Ok(metadata) => metadata,
						Err(_) => continue,
					};
				tracks_vec.push(track);
			}
		}
	}
	Ok(tracks_vec)
}

fn get_track_metadata(file_path: &str) -> std::io::Result<Track> {
	let path = Path::new(file_path);
	let tagged_file = Probe::open(&path).unwrap().read();

	let mut artist = String::from("Unknown Artist");
	let mut title = String::from("Unknown Title");
	let mut album = String::from("Unknown Album");
	let mut track_num: i32 = 0;

	if let Some(tag) = tagged_file.unwrap().primary_tag() {
		if let Some(t) = tag.get_string(&ItemKey::TrackTitle) {
			title = t.to_string();
		}
		if let Some(n) = tag.get_string(&ItemKey::TrackNumber) {
			track_num = match n.parse::<i32>() {
				Ok(n) => n,
				Err(_) => 0,
			}
		}
		if let Some(a) = tag.get_string(&ItemKey::TrackArtist) {
			artist = a.to_string();
		}
		if let Some(al) = tag.get_string(&ItemKey::AlbumTitle) {
			album = al.to_string();
		}
	}

	Ok(Track::new(
		artist,
		title,
		track_num,
		file_path.to_string(),
		album,
	))
}
