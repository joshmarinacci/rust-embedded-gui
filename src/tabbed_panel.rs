use crate::geom::{Bounds, Insets};
use crate::input::OutputAction;
use crate::scene::Scene;
use crate::toggle_group::{input_toggle_group, make_toggle_group};
use crate::view::Flex::Resize;
use crate::view::{Flex, View, ViewId};
use crate::LayoutEvent;
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
        let res = input_toggle_group(e);
        if let Some(action) = res {
            info!("action is {:?}", action);
            let Some(container) = e.scene.get_parent_for_view(e.target) else {
                return None;
            };
            let container = container.clone();

            // hide all the children
            for kid in e.scene.get_children_ids(&container) {
                if kid.to_string() != "tabs" {
                    e.scene.hide_view(&kid);
                }
            }

            // then make the newly selected one visible
            match action {
                OutputAction::Command(cmd) => {
                    let panel_name = if let Some(state) =
                        e.scene.get_view_state::<LayoutPanelState>(&container)
                    {
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
                _ => {}
            }
        }
        return None;
    });
    scene.add_view_to_parent(tabs, name);

    View {
        name: name.clone(),
        bounds: Bounds::new(10, 10, 320 - 20, 180),
        h_flex: Flex::Intrinsic,
        v_flex: Flex::Intrinsic,
        draw: Some(|e| {
            e.ctx.fill_rect(&e.view.bounds, &e.theme.panel.fill);
            e.ctx.stroke_rect(&e.view.bounds, &e.theme.panel.text);
        }),
        state: Some(Box::new(state)),
        layout: Some(layout_tabbed_panel),
        ..Default::default()
    }
}

pub fn layout_tabbed_panel(pass: &mut LayoutEvent) {
    if let Some(view) = pass.scene.get_view_mut(&pass.target) {
        // layout self
        if view.h_flex == Resize {
            view.bounds.size.w = pass.space.w;
        }
        if view.v_flex == Resize {
            view.bounds.size.h = pass.space.h;
        }

        // layout tabs
        let space = view.bounds.size.clone();
        let tabs_id: ViewId = "tabs".into();
        pass.layout_child(&tabs_id, space);

        // layout content panels
        if let Some(view) = pass.scene.get_view(&tabs_id) {
            let insets = Insets::new(view.bounds.size.h, 1, 1, 1);
            for kid in &pass.scene.get_children_ids(&pass.target) {
                if kid == &tabs_id {
                    continue;
                }
                pass.layout_child(kid, space - insets);
                if let Some(view) = pass.scene.get_view_mut(kid) {
                    view.bounds.position.y = insets.top;
                    view.bounds.position.x = insets.left;
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::geom::{Bounds, Size};
    use crate::layouts::{layout_hbox, layout_std_panel, layout_vbox};
    use crate::panel::make_panel;
    use crate::scene::{layout_scene, Scene};
    use crate::tabbed_panel::layout_tabbed_panel;
    use crate::test::MockDrawingContext;
    use crate::toggle_group::layout_toggle_group;
    use crate::view::Align::{Center, End, Start};
    use crate::view::Flex::Resize;
    use crate::view::{Flex, View, ViewId};

    #[test]
    fn test_complex_tabbed_panels() {
        let mut scene = Scene::new();
        // tab panel test
        let tabbed_panel: ViewId = "tabbed_panel_id".into();
        let tabs: ViewId = "tabs".into();
        {
            let mut tabbed_panel_view: View = View {
                name: tabbed_panel.clone(),
                ..Default::default()
            };
            tabbed_panel_view.h_flex = Flex::Resize;
            tabbed_panel_view.v_flex = Flex::Resize;
            tabbed_panel_view.layout = Some(layout_tabbed_panel);
            scene.add_view_to_parent(tabbed_panel_view, &scene.root_id());

            // tab panel tabs has intrisic height but flex width
            let mut tabbed_panel_tabs: View = View {
                name: tabs.clone(),
                ..Default::default()
            };
            tabbed_panel_tabs.h_flex = Flex::Resize;
            tabbed_panel_tabs.v_flex = Flex::Intrinsic;
            tabbed_panel_tabs.layout = Some(layout_toggle_group);
            scene.add_view_to_parent(tabbed_panel_tabs, &tabbed_panel);
        }

        // has three tabs contents
        // tab panel contents are panels that grow to fill space
        // first tab panel is an hbox with three buttons vertically centered and left aligned
        let tab1: ViewId = "tab1".into();
        {
            let view = make_panel(&tab1)
                .with_layout(Some(layout_hbox))
                .with_flex(Resize, Resize);
            scene.add_view_to_parent(view, &tabbed_panel);

            let b1: ViewId = "tab1_button1".into();
            {
                let mut button = crate::layouts::tests::make_standard_view(&b1);
                button.title = "abc".into();
                button.v_align = Start;
                button.layout = Some(crate::layouts::tests::layout_button);
                scene.add_view_to_parent(button, &tab1);
            }

            let b2: ViewId = "tab1_button2".into();
            {
                let mut button = crate::layouts::tests::make_standard_view(&b2);
                button.v_align = Center;
                button.layout = Some(crate::layouts::tests::layout_button);
                scene.add_view_to_parent(button, &tab1);
            }

            let b3: ViewId = "tab1_button3".into();
            {
                let mut button = crate::layouts::tests::make_standard_view(&b3);
                button.v_align = End;
                button.layout = Some(crate::layouts::tests::layout_button);
                scene.add_view_to_parent(button, &tab1);
            }
        }

        // second tab panel is a vbox with three buttons horizontally centered
        // and the first one takes all vertical space and horizontal space
        let tab2: ViewId = "tab2".into();
        {
            let view = make_panel(&tab2)
                .with_layout(Some(layout_vbox))
                .with_flex(Resize, Resize);
            scene.add_view_to_parent(view, &tabbed_panel);

            let b1: ViewId = "tab2_button1".into();
            {
                let mut b1_view = crate::layouts::tests::make_standard_view(&b1);
                b1_view.h_align = Start;
                b1_view.h_flex = Flex::Resize;
                b1_view.v_flex = Flex::Resize;
                b1_view.title = "b11".into();
                b1_view.layout = Some(layout_std_panel);
                scene.add_view_to_parent(b1_view, &tab2);
            }

            let b2: ViewId = "tab2_button2".into();
            {
                let mut b2_view = crate::layouts::tests::make_standard_view(&b2);
                b2_view.h_align = End;
                b2_view.h_flex = Flex::Intrinsic;
                b2_view.v_flex = Flex::Intrinsic;
                b2_view.title = "b11".into();
                b2_view.layout = Some(crate::layouts::tests::layout_button);
                scene.add_view_to_parent(b2_view, &tab2);
            }
        }

        // third tab panel lets children be absolutely positioned and sizes self to the center with a fixed width and height of 100
        let tab3: ViewId = "tab3".into();
        {
            let mut view = crate::layouts::tests::make_standard_view(&tab3);
            view.h_flex = Flex::Resize;
            view.v_flex = Flex::Resize;
            view.layout = Some(layout_std_panel);
            scene.add_view_to_parent(view, &tabbed_panel);
        }

        // tab panel is 200 x 200px to fill the root
        let scene_size = Size::new(200, 200);

        let theme = MockDrawingContext::make_mock_theme();
        layout_scene(&mut scene, &theme);
        scene.dump();
        assert_eq!(
            scene.get_view_bounds(&scene.root_id()),
            Some(Bounds::new(0, 0, 200, 200))
        );
        assert_eq!(
            scene.get_view_bounds(&tabbed_panel),
            Some(Bounds::new(0, 0, 200, 200))
        );
        assert_eq!(
            scene.get_view_bounds(&tabs),
            Some(Bounds::new(0, 0, 200, 20))
        );
        assert_eq!(
            scene.get_view_bounds(&tab1),
            Some(Bounds::new(1, 20, 198, 179))
        );
        assert_eq!(
            scene.get_view_bounds(&tab2),
            Some(Bounds::new(1, 20, 198, 179))
        );
        assert_eq!(
            scene.get_view_bounds(&tab3),
            Some(Bounds::new(1, 20, 198, 179))
        );

        assert_eq!(
            scene.get_view_bounds(&"tab1_button1".into()),
            Some(Bounds::new(0, 0, 30, 10))
        );
        assert_eq!(
            scene.get_view_bounds(&"tab2_button1".into()),
            Some(Bounds::new(0, 0, 198, 169))
        );
        assert_eq!(
            scene.get_view_bounds(&"tab2_button2".into()),
            Some(Bounds::new(168, 169, 30, 10))
        );
    }
}
