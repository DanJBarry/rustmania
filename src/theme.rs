use crate::{
    music::Music,
    notefield::Notefield,
    player_config::NoteLayout,
    screen::{Element, Screen},
    timingdata::{GameplayInfo, TimingData},
};
use serde_derive::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Deserialize, Serialize)]
pub struct ScreenBuilder {
    pub elements: Vec<ElementType>,
}

impl ScreenBuilder {
    pub fn build(&self, resources: &Resources) -> Screen {
        let element_list = self
            .elements
            .iter()
            .map(|element| element.build(resources))
            .collect();
        Screen::new(element_list)
    }
}

#[derive(Deserialize, Serialize)]
pub enum ElementType {
    MUSIC(usize, usize),
    NOTEFIELD(usize, usize),
    TEXT(usize, usize, usize),
}

impl ElementType {
    pub fn build(&self, resources: &Resources) -> Box<dyn Element> {
        match self {
            Self::MUSIC(rate, name) => Box::new(Music::new(
                resources.floats[*rate],
                resources.paths[*name].clone(),
            )),
            Self::NOTEFIELD(layout, timing_data) => Box::new(Notefield::new(
                resources.layouts[*layout].clone(),
                &resources.notes[*timing_data],
                600,
            )),
            Self::TEXT(contents, x_pos, y_pos) => Box::new(crate::text::TextBox::new(
                resources.strings[*contents].clone(),
                [
                    resources.floats[*x_pos] as f32,
                    resources.floats[*y_pos] as f32,
                ],
                0,
            )),
        }
    }
}

pub struct Resources {
    notes: Vec<TimingData<GameplayInfo>>,
    paths: Vec<PathBuf>,
    layouts: Vec<NoteLayout>,
    floats: Vec<f64>,
    #[allow(clippy::used_underscore_binding)]
    _integers: Vec<i64>,
    strings: Vec<String>,
}

impl Resources {
    pub fn _new() -> Self {
        Self {
            notes: vec![],
            paths: vec![],
            layouts: vec![],
            floats: vec![],
            _integers: vec![],
            strings: vec![],
        }
    }
    pub fn from(
        notes: Vec<TimingData<GameplayInfo>>,
        paths: Vec<PathBuf>,
        layouts: Vec<NoteLayout>,
        floats: Vec<f64>,
        integers: Vec<i64>,
        strings: Vec<String>,
    ) -> Self {
        Self {
            notes,
            paths,
            layouts,
            floats,
            _integers: integers,
            strings,
        }
    }
}
