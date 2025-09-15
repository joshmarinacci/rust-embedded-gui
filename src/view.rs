use crate::geom::Bounds;
use crate::{DrawFn, DrawFn2, InputFn, LayoutFn};
use alloc::boxed::Box;
use alloc::string::String;
use core::any::Any;

#[derive(Debug)]
pub struct View<C, F> {
    pub name: String,
    pub title: String,
    pub bounds: Bounds,
    pub visible: bool,
    pub draw: Option<DrawFn<C, F>>,
    pub input: Option<InputFn<C, F>>,
    pub state: Option<Box<dyn Any>>,
    pub layout: Option<LayoutFn<C, F>>,
    pub draw2: Option<DrawFn2<C, F>>,
}

impl<C, F> View<C, F> {
    pub fn position_at(mut self, x: i32, y: i32) -> View<C, F> {
        self.bounds.x = x;
        self.bounds.y = y;
        self
    }
    pub fn hide(mut self) -> View<C, F> {
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
