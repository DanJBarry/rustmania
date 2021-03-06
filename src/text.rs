use crate::screen::{Element, Message, Resource};
use ggez::{
    event::KeyCode,
    graphics::{self, DrawParam},
    Context, GameError,
};
use std::time::Instant;

pub struct TextBox {
    image: graphics::Text,
    _contents: String,
    position: [f32; 2],
    _size: u32,
}

impl TextBox {
    pub fn new(contents: String, position: [f32; 2], size: u32) -> Self {
        Self {
            image: graphics::Text::new(contents.clone()),
            _contents: contents,
            position,
            _size: size,
        }
    }
}

impl Element for TextBox {
    fn run(&mut self, context: &mut Context, _time: Option<i64>) -> Result<Message, GameError> {
        graphics::draw(context, &self.image, DrawParam::new().dest(self.position))?;
        Ok(Message::None)
    }
    fn start(&mut self, _time: Option<Instant>) -> Result<Message, GameError> {
        Ok(Message::None)
    }
    fn finish(&mut self) -> Option<Resource> {
        None
    }
    fn handle_event(&mut self, _key: KeyCode, _time: Option<i64>, _key_down: bool) {}
}
