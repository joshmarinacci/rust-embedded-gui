use crate::geom::Bounds;
use crate::{DrawFn, InputFn, LayoutFn};
use alloc::borrow::Cow;
use alloc::boxed::Box;
use alloc::string::{String, ToString};
use core::any::Any;
use core::fmt::{Display, Formatter};

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct ViewId(Cow<'static, str>);

impl ViewId {
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl ViewId {
    pub const fn new(id: &'static str) -> Self {
        ViewId(Cow::Borrowed(id))
    }
    pub fn make(id: String) -> Self {
        ViewId(Cow::Owned(id))
    }
}
impl Display for ViewId {
    fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
        write!(f, "{}", self.0)
    }
}

#[derive(PartialEq, Debug, Copy, Clone)]
pub enum Flex {
    Intrinsic,
    Resize,
}
#[derive(Copy, Clone, PartialEq, Debug)]
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
    pub fn with_name(mut self, name: ViewId) -> View {
        self.name = name;
        self
    }
    pub fn with_bounds(mut self, bounds: Bounds) -> View {
        self.bounds = bounds;
        self
    }
    pub fn with_layout(mut self, layout: Option<LayoutFn>) -> View {
        self.layout = layout;
        self
    }
    pub fn with_state(mut self, state: Option<Box<dyn Any>>) -> View {
        self.state = state;
        self
    }

    pub fn with_flex(mut self, h_flex: Flex, v_flex: Flex) -> View {
        self.h_flex = h_flex;
        self.v_flex = v_flex;
        self
    }
}

impl View {
    pub fn position_at(mut self, x: i32, y: i32) -> View {
        self.bounds.position.x = x;
        self.bounds.position.y = y;
        self
    }
    pub fn with_size(mut self, w: i32, h: i32) -> View {
        self.bounds.size.w = w;
        self.bounds.size.h = h;
        self
    }
    pub fn with_visible(mut self, visible: bool) -> View {
        self.visible = visible;
        self
    }
    pub fn hide(&mut self) {
        self.visible = false;
    }
    pub fn get_state<T: 'static>(&mut self) -> Option<&mut T> {
        if let Some(view) = &mut self.state {
            return view.downcast_mut::<T>();
        }
        None
    }

    pub fn with_draw_fn(mut self, draw: Option<DrawFn>) -> View {
        self.draw = draw;
        self
    }
}

impl Default for View {
    fn default() -> Self {
        let id: ViewId = ViewId::new("noname");
        View {
            name: id.clone(),
            title: id.to_string(),
            bounds: Default::default(),

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
