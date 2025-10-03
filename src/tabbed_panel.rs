use crate::geom::Bounds;
use crate::layouts::layout_tabbed_panel;
use crate::scene::Scene;
use crate::toggle_group::{input_toggle_group, make_toggle_group};
use crate::view::{Flex, View, ViewId};
use crate::Action;
use alloc::boxed::Box;
use alloc::string::{String, ToString};
use alloc::vec;
use alloc::vec::Vec;
use hashbrown::HashMap;
use log::info;

pub struct LayoutPanelState {
    data: Vec<String>,
    contents: HashMap<String, ViewId>,
}

impl LayoutPanelState {
    pub fn register_panel(&mut self, tab_name: &str, content_id: &ViewId) {
        info!("registering panel {tab_name} with content {content_id}");
        self.data.push(tab_name.into());
        self.contents.insert(tab_name.into(), content_id.clone());
    }
}

pub fn make_tabbed_panel(
    name: &ViewId,
    data: Vec<&str>,
    selected: usize,
    scene: &mut Scene,
) -> View {
    let state = LayoutPanelState {
        data: vec![],
        contents: HashMap::new(),
    };

    let tabs_id = ViewId::new("tabs");
    let mut tabs = make_toggle_group(&tabs_id, data, selected);
    tabs.input = Some(|e| {
        info!("got input");
        let container = ViewId::new("tabbed-panel");
        let res = input_toggle_group(e);
        if let Some(action) = res {
            info!("action is {:?}", action);

            // hide all the children
            for kid in e.scene.get_children_ids(&container) {
                if kid.as_str() != "tabs" {
                    e.scene.hide_view(&kid);
                }
            }

            // then make the newly selected one visible
            match action {
                Action::Generic => {}
                Action::Command(cmd) => {
                    let panel_name = if let Some(state) = e.scene.get_view_state::<LayoutPanelState>(&container) {
                        if let Some(panel) = state.contents.get(&cmd) {
                            Some(panel.clone())
                        } else {
                            None
                        }
                    } else {
                        None
                    };
                    info!("found the panel name {panel_name:?}");
                    if let Some(panel) = panel_name {
                        info!("setting the panel {}", panel);
                        e.scene.show_view(&panel);
                    }
                }
            }
        }
        return None;
    });
    scene.add_view_to_parent(tabs, &name);

    View {
        name: name.clone(),
        bounds: Bounds::new(10, 10, 320 - 20, 180),
        h_flex: Flex::Intrinsic,
        v_flex: Flex::Intrinsic,
        draw: Some(|e| {
            e.ctx.fill_rect(&e.view.bounds, &e.theme.bg);
            e.ctx.stroke_rect(&e.view.bounds, &e.theme.fg);
        }),
        state: Some(Box::new(state)),
        layout: Some(layout_tabbed_panel),
        ..Default::default()
    }
}
