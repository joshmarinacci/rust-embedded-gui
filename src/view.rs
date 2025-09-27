use crate::geom::{Bounds, Insets, Size};
use crate::{DrawFn, InputFn, LayoutFn, Theme};
use alloc::boxed::Box;
use alloc::string::String;
use core::any::Any;

pub type ViewId = String;
#[derive(PartialEq, Debug)]
pub enum Flex {
    Intrinsic,
    Resize,
}
#[derive(PartialEq, Debug)]
pub enum Align {
    Start,
    Center,
    End,
}

#[derive(Debug)]
pub struct View {
    pub name: ViewId,
    pub title: String,
    pub bounds: Bounds,
    pub padding: Insets,

    pub v_flex: Flex,
    pub h_flex: Flex,
    pub h_align: Align,
    pub v_align: Align,

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

impl Default for View {
    fn default() -> Self {
        let id: ViewId = "noname".into();
        View {
            name: id.clone(),
            title: id.clone(),
            bounds: Default::default(),
            padding: Default::default(),

            h_flex: Flex::Intrinsic,
            v_flex: Flex::Intrinsic,
            h_align: Align::Center,
            v_align: Align::Center,

            visible: true,
            input: None,
            state: None,
            layout: None,
            draw: None,
        }
    }
}

