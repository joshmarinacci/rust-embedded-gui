use crate::geom::Bounds;
use crate::{DrawFn, InputFn, LayoutFn};
use alloc::boxed::Box;
use alloc::string::String;
use core::any::Any;

#[derive(Debug)]
pub struct View {
    pub name: String,
    pub title: String,
    pub bounds: Bounds,
    pub visible: bool,
    pub input: Option<InputFn>,
    pub state: Option<Box<dyn Any>>,
    pub layout: Option<LayoutFn>,
    pub draw: Option<DrawFn>,
}

impl View {
    pub fn position_at(mut self, x: i32, y: i32) -> View {
        self.bounds.position.x= x;
        self.bounds.position.y= y;
        self
    }
    pub fn with_size(mut self, w: i32, h: i32) -> View {
        self.bounds.size.w= w;
        self.bounds.size.h= h;
        self
    }
    pub fn hide(mut self) -> View {
        self.visible = false;
        self
    }
    pub fn get_state<T: 'static>(&mut self) -> Option<&mut T> {
        if let Some(view) = &mut self.state {
            return view.downcast_mut::<T>();
        }
        None
    }
}
