#![allow(deprecated)]
#![warn(clippy::pedantic)]
#![allow(
    clippy::cast_possible_truncation,
    clippy::cast_possible_wrap,
    clippy::cast_precision_loss,
    clippy::cast_sign_loss,
    clippy::module_name_repetitions
)]

mod difficulty_calc;
mod gamestate;
mod music;
mod notedata;
mod notefield;
mod player_config;
mod screen;
mod song_loader;
mod text;
mod timingdata;

use crate::{
    gamestate::GameState,
    notedata::NoteType,
    player_config::NoteSkin,
    screen::{
        ElementMap, ElementType, ResourceMap, ResourceType, Resources, ScreenBuilder, ScriptMap,
    },
};
use clap::{crate_authors, crate_version, App, Arg};
use ggez::{filesystem::mount, graphics::Rect, ContextBuilder};
use log::{debug, info};
use num_rational::Rational32;
use rand::seq::SliceRandom;
use std::{
    cmp::Ordering,
    fs::{File, OpenOptions},
    io::Read,
    path::PathBuf,
    str::from_utf8,
    time::Instant,
};

const NOTEFIELD_SIZE: usize = 4;

fn set_up_logging() -> Result<(), fern::InitError> {
    fern::Dispatch::new()
        .format(|out, message, _record| out.finish(format_args!("{}", message)))
        .chain(
            OpenOptions::new()
                .write(true)
                .create(true)
                .open("log.txt")?,
        )
        .apply()?;
    Ok(())
}

fn sprite_finder(
    _measure: usize,
    _row_time: f64,
    row_alignment: Rational32,
    note_type: NoteType,
    _column: usize,
) -> Rect {
    match note_type {
        NoteType::Tap | NoteType::Hold => {
            let &division = (row_alignment * 4).denom();
            match division {
                1 => Rect::new(0.0, 0.0, 1.0, 0.125),
                2 => Rect::new(0.0, 0.125, 1.0, 0.125),
                3 => Rect::new(0.0, 0.25, 1.0, 0.125),
                4 => Rect::new(0.0, 0.375, 1.0, 0.125),
                6 => Rect::new(0.0, 0.5, 1.0, 0.125),
                8 => Rect::new(0.0, 0.625, 1.0, 0.125),
                12 => Rect::new(0.0, 0.75, 1.0, 0.125),
                _ => Rect::new(0.0, 0.875, 1.0, 0.125),
            }
        }
        _ => Rect::new(0.0, 0.0, 1.0, 1.0),
    }
}

mod callbacks {
    use crate::screen::Resource;
    use crate::timingdata::TimingColumn;

    pub fn map_to_string(resource: Option<Resource>) -> Option<Resource> {
        resource.map(|resource| match resource {
            Resource::Replay(replay) => Resource::String(
                (replay
                    .iter()
                    .map(|column| column.current_points(1.0))
                    .sum::<f64>()
                    / replay.iter().map(TimingColumn::max_points).sum::<f64>()
                    * 100.0)
                    .to_string(),
            ),
            _ => Resource::String("".to_owned()),
        })
    }
}

fn main() {
    let mut rng = rand::thread_rng();
    let matches = App::new("Rustmania")
        .author(crate_authors!())
        .version(crate_version!())
        .about("A rhythm game in the vein of Stepmania and Etterna, currently in very early stages of development.")
        .args(&[
            Arg::with_name("SimFile")
                .help("The path to your .sm file.")
                .index(1)
                .required(false),
            Arg::with_name("Rate")
                .help("The rate of the music.")
                .index(2)
                .required(false),
            Arg::with_name("NoteSkin")
                .help("The name of your NoteSkin folder.")
                .index(3)
                .required(false),
            Arg::with_name("Theme")
                .help("The path to your lua theme file.")
                .index(4)
                .required(false),
        ])
        .after_help("Licenced under MIT.")
        .get_matches();

    set_up_logging().expect("Failed to setup logging");

    let songs_folder = match matches.value_of("SimFile") {
        Some(value) => format!("Songs/{}", value),
        None => String::from("Songs"),
    };

    let (simfile_folder, difficulty, notedata) = {
        let start_time = Instant::now();
        let notedata_list = song_loader::load_songs_folder(songs_folder);
        let duration = Instant::now() - start_time;
        info!("Found {} total songs", notedata_list.len());
        let mut notedata_list = notedata_list
            .into_iter()
            .filter_map(|x| x)
            .collect::<Vec<_>>();
        info!("Of which, {} loaded", notedata_list.len());
        info!(
            "This took {}.{} seconds",
            duration.as_secs(),
            duration.subsec_millis()
        );
        notedata_list.sort_by(|a, b| (a.1).0.partial_cmp(&(b.1).0).unwrap_or(Ordering::Less));
        notedata_list
            .iter()
            .for_each(|x| info!("{:?}, {}", (x.1).1.meta.title, (x.1).0));
        let (simfile_path, (difficulty, notedata)) = notedata_list
            .choose(&mut rng)
            .expect("Failed to select chart from cache")
            .clone();
        let simfile_folder = String::from(
            simfile_path
                .parent()
                .expect("No parent folder for selected file")
                .as_os_str()
                .to_str()
                .expect("failed to parse path"),
        );
        (simfile_folder, difficulty, notedata)
    };
    println!(
        "Selected Song is: {}",
        notedata.meta.title.clone().unwrap_or_default()
    );
    println!("With difficulty: {}", difficulty);

    let noteskin = matches.value_of("NoteSkin").unwrap_or("Default");

    let music_rate = if let Some(Ok(rate)) = matches.value_of("Rate").map(str::parse) {
        rate
    } else {
        1.0
    };

    let (context, events_loop) = &mut ContextBuilder::new("rustmania", "ixsetf")
        .add_resource_path("")
        .window_setup(ggez::conf::WindowSetup {
            title: "Rustmania".to_string(),
            ..ggez::conf::WindowSetup::default()
        })
        .build()
        .expect("Failed to build context");

    let default_note_skin = NoteSkin::from_path(&format!("Noteskins\\{}", noteskin), context)
        .expect("Could not open default noteskin");

    let p1_options = player_config::PlayerOptions::new(200, 125, 0.8, true, (-128.0, 383.0));
    let p2_options = player_config::PlayerOptions::new(600, 125, 1.1, false, (-128.0, 383.0));

    let p1_layout = player_config::NoteLayout::new(&default_note_skin, 600, p1_options);
    let p2_layout = player_config::NoteLayout::new(&default_note_skin, 600, p2_options);

    let notes = timingdata::TimingData::from_notedata(&notedata, sprite_finder, music_rate);

    let resources = Resources::from(
        notes,
        vec![PathBuf::from(format!(
            "{}/{}",
            simfile_folder,
            notedata.meta.music_path.expect("No music path specified")
        ))],
        vec![p1_layout, p2_layout],
        vec![music_rate, 0.0, 12.0],
        vec![600],
        vec![
            notedata.meta.title.expect("Needs a title").clone(),
            String::from("Results screen placeholder text"),
        ],
        vec![],
        vec![],
    );

    let gameplay_screen = match matches.value_of("Theme") {
        Some(value) => {
            // This currently is not getting the music rate so the theme will have incorrect behavior
            // if the rate specified in the theme is different than the rate passed in through the CLI
            let mut theme =
                File::open(format!("Themes/Default/{}", value)).expect("Can not find theme file");
            let mut theme_string = vec![];
            theme
                .read_to_end(&mut theme_string)
                .expect("Could not read theme file completely");
            serde_yaml::from_str(
                from_utf8(&theme_string).expect("Can not parse theme file as string"),
            )
            .expect("Can not parse theme file as YAML")
        }
        None => ScreenBuilder {
            elements: vec![
                ElementType::NOTEFIELD(0, 0, 0),
                ElementType::NOTEFIELD(1, 0, 0),
                ElementType::MUSIC(0, 0),
                ElementType::TEXT(0, 1, 2),
            ],
        },
    };

    let results_screen = ScreenBuilder {
        elements: vec![ElementType::TEXT(2, 1, 2)],
    };

    if let Ok(manifest_dir) = std::env::var("CARGO_MANIFEST_DIR") {
        let mut path = PathBuf::from(manifest_dir);
        path.push("resources");
        mount(context, &path, true);
    }

    let mut gamestate = GameState::from(
        vec![
            (
                gameplay_screen,
                vec![
                    ResourceMap::Element(ElementMap {
                        element_index: 0,
                        resource_index: 0,
                    }),
                    ResourceMap::Script(ScriptMap {
                        resource_type: ResourceType::Replay,
                        resource_index: 0,
                        script_index: 0,
                    }),
                ],
            ),
            (results_screen, vec![]),
        ],
        resources,
        vec![callbacks::map_to_string],
    );
    if let Err(e) = ggez::event::run(context, events_loop, &mut gamestate) {
        debug!("Error: {}", e);
    } else {
        debug!("Exit successful.");
    }
}
