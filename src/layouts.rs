use log::info;
use Flex::Intrinsic;
use crate::geom::{Insets, Size};
use crate::LayoutEvent;
use crate::view::Align::{Center, End, Start};
use crate::view::{Flex, ViewId};
use crate::view::Flex::Resize;

pub fn layout_vbox(pass: &mut LayoutEvent) {
    let Some(parent) = pass.scene.get_view_mut(&pass.target) else {
        info!("view not found!");
        return;
    };
    let padding = parent.padding.clone();
    let available_space: Size = pass.space - parent.padding;

    // get the intrinsic children
    let fixed_kids = pass
        .scene
        .get_children_ids_filtered(&pass.target, |v| v.v_flex == Intrinsic);
    // lay out the intrinsic children
    for kid in &fixed_kids {
        pass.layout_child(kid, available_space);
    }

    // calculate total used height
    let kids_sum = fixed_kids.iter().fold(0, |a, id| {
        return if let Some(view) = pass.scene.get_view_mut(id) {
            view.bounds.size.h + a
        } else {
            a
        };
    });
    let vert_leftover = (pass.space - padding).h - kids_sum;

    // get the flex children
    let flex_kids = pass
        .scene
        .get_children_ids_filtered(&pass.target, |v| v.v_flex == Flex::Resize);
    if flex_kids.len() > 0 {
        let flex_space = Size {
            w: pass.space.w - padding.left - padding.right,
            h: vert_leftover / (flex_kids.len() as i32),
        };
        for kid in flex_kids {
            pass.layout_child(&kid, flex_space);
        }
    }

    let mut y = padding.top as i32;
    let all_kids = pass.scene.get_children_ids(&pass.target);
    for kid in all_kids {
        let avail_w = pass.space.w - padding.left - padding.right;
        if let Some(kid) = pass.scene.get_view_mut(&kid) {
            kid.bounds.position.x = match &kid.h_align {
                Start => padding.left + (avail_w - kid.bounds.size.w) * 0,
                Center => padding.left + (avail_w - kid.bounds.size.w) / 2,
                End => padding.left + (avail_w - kid.bounds.size.w),
            };
            kid.bounds.position.y = y;
            y += kid.bounds.size.h;
        }
    }
    // layout self
    if let Some(view) = pass.scene.get_view_mut(&pass.target) {
        view.bounds.size = pass.space.clone();
    }
}

pub fn layout_hbox_2(pass: &mut LayoutEvent) {
    let Some(parent) = pass.scene.get_view_mut(&pass.target) else {
        return;
    };
    // layout self
    if parent.v_flex == Resize {
        parent.bounds.size.h = pass.space.h
    }
    if parent.h_flex == Resize {
        parent.bounds.size.w = pass.space.w
    }

    let space = parent.bounds.size;
    let padding = parent.padding.clone();
    let available_space: Size = pass.space - padding;

    // get the fixed children
    let fixed_kids = pass
        .scene
        .get_children_ids_filtered(&pass.target, |v| v.h_flex == Intrinsic);

    // layout the fixed width children
    for kid in &fixed_kids {
        pass.layout_child(kid, available_space);
    }

    // calc the total width of the fixed kids
    let kids_sum: i32 = fixed_kids
        .iter()
        .map(|id| pass.scene.get_view(id))
        .flatten()
        .fold(0, |a, v| v.bounds.size.w + a);
    let avail_horizontal_space = (space - padding).h - kids_sum;

    // get the flex children
    let flex_kids = pass
        .scene
        .get_children_ids_filtered(&pass.target, |v| v.h_flex == Flex::Resize);
    // if there are any flex children
    if flex_kids.len() > 0 {
        // split the leftover space
        let flex_space = Size {
            w: avail_horizontal_space / (flex_kids.len() as i32),
            h: pass.space.h - padding.top - padding.bottom,
        };
        // layout the flex children
        for kid in &flex_kids {
            pass.layout_child(kid, flex_space);
        }
    }

    // now position all children
    // let all_kids = pass.scene.get_children(&pass.name);
    let avail_h = space.h - padding.top - padding.bottom;
    let mut x = padding.left;
    for kid in pass.scene.get_children_ids(&pass.target) {
        if let Some(kid) = pass.scene.get_view_mut(&kid) {
            kid.bounds.position.x = x;
            x += kid.bounds.size.w;
            kid.bounds.position.y = match &kid.v_align {
                Start => (avail_h - kid.bounds.size.h) * 0,
                Center => (avail_h - kid.bounds.size.h) / 2,
                End => (avail_h - kid.bounds.size.h),
            } + padding.top;
        }
    }

}

fn layout_button(layout: &mut LayoutEvent) {
    if let Some(view) = layout.scene.get_view_mut(&layout.target) {
        view.bounds.size = Size::new((view.title.len() * 10) as i32, 10) + view.padding;
    }
}
pub fn layout_std_panel(pass: &mut LayoutEvent) {
    if let Some(view) = pass.scene.get_view_mut(&pass.target) {
        if view.v_flex == Resize {
            view.bounds.size.h = pass.space.h;
        }
        if view.h_flex == Resize {
            view.bounds.size.w = pass.space.w;
        }
        let space = view.bounds.size.clone() - view.padding;
        pass.layout_all_children(&pass.target.clone(),space);
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
            let insets = Insets::new(view.bounds.size.h,0,0,0);
            for kid in &pass.scene.get_children_ids(&pass.target) {
                if kid == &tabs_id {
                    continue;
                }
                pass.layout_child(kid,space - insets);
                if let Some(view) = pass.scene.get_view_mut(kid) {
                    view.bounds.position.y = insets.top;
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::geom::{Bounds, Insets, Point, Size};
    use crate::layouts::{layout_button, layout_hbox_2, layout_std_panel, layout_tabbed_panel, layout_vbox};
    use crate::scene::{layout_scene, Scene};
    use crate::test::MockDrawingContext;
    use crate::toggle_group::layout_toggle_group;
    use crate::view::{Align, Flex, View, ViewId};
    use crate::view::Align::{Center, End, Start};

    #[test]
    fn test_button() {
        let button = View {
            name: "button1".into(),
            title: "abc".into(),
            layout: Some(layout_button),
            padding: Insets::new_same(10),
            ..Default::default()
        };

        let theme = MockDrawingContext::make_mock_theme();
        let mut scene = Scene::new();
        scene.add_view_to_parent(button, &scene.root_id());
        layout_scene(&mut scene, &theme);
        // size = 3 letters x 10x10 font + 10px padding
        assert_eq!(
            view_bounds(&scene, &"button1".into()).size,
            Size::new(3 * 10 + 20, 10 + 20),
            "button size is wrong"
        );
    }

    fn view_bounds(scene: &Scene, name: &ViewId) -> Bounds {
        if let Some(view) = scene.get_view(name) {
            view.bounds
        } else {
            Bounds::new(-99, -99, -99, -99)
        }
    }

    #[test]
    fn test_vbox() {
        let mut scene = Scene::new();
        let parent_id: ViewId = "parent".into();
        let parent_view = View {
            name: parent_id.clone(),
            title: "parent".into(),
            padding: Insets::new_same(10),
            bounds: Bounds {
                position: Point::new(-99, -99),
                size: Size::new(100, 100),
            },
            h_flex: Flex::Resize,
            v_flex: Flex::Resize,
            h_align: Align::Center,
            v_align: Align::Center,
            layout: Some(layout_vbox),
            ..Default::default()
        };

        let child1_id: ViewId = "child1".into();
        scene.add_view_to_parent(View {
            name: child1_id.clone(),
            title: "ch1".into(),
            h_align: Align::Start,
            layout: Some(layout_button),
            ..Default::default()
        }, &parent_id);

        let child2_id: ViewId = "child2".into();
        scene.add_view_to_parent(View {
            name: child2_id.clone(),
            title: "ch2".into(),
            h_align: Align::Center,
            layout: Some(layout_button),
            ..Default::default()
        }, &parent_id);

        let child3_id: ViewId = "child3".into();
        scene.add_view_to_parent(View {
            name: child3_id.clone(),
            title: "ch3".into(),
            h_align: Align::End,
            layout: Some(layout_button),
            ..Default::default()
        }, &parent_id);

        let child4_id: ViewId = "child4".into();
        scene.add_view_to_parent(View {
            name: child4_id.clone(),
            title: "ch4".into(),
            h_flex: Flex::Resize,
            v_flex: Flex::Resize,
            layout: Some(layout_std_panel),
            ..Default::default()
        }, &parent_id);

        scene.add_view_to_parent(parent_view, &scene.root_id());

        let theme = MockDrawingContext::make_mock_theme();

        layout_scene(&mut scene, &theme);
        if let Some(view) = scene.get_view_mut(&parent_id) {
            assert_eq!(view.name, parent_id);
            // confirm position wasn't modified at all
            assert_eq!(view.bounds.position, Point::new(-99, -99));
            // size = scene size of 200x200
            assert_eq!(view.bounds.size, Size::new(200, 200));
            // left align
            if let Some(view) = scene.get_view(&child1_id) {
                assert_eq!(view.bounds.position, Point::new(10, 10));
                assert_eq!(view.bounds.size, Size::new(30, 10));
            }
            // center align
            if let Some(view) = scene.get_view(&child2_id) {
                assert_eq!(view.bounds.position, Point::new(10 + (180 - 30) / 2, 20));
                assert_eq!(view.bounds.size, Size::new(30, 10));
            }
            // right align
            if let Some(view) = scene.get_view(&child3_id) {
                assert_eq!(view.bounds.position, Point::new(10 + (180 - 30), 30));
                assert_eq!(view.bounds.size, Size::new(30, 10));
            }
            // should fill rest of the space
            assert!(scene.has_view(&child4_id));
            if let Some(view) = scene.get_view(&child4_id) {
                assert_eq!(view.bounds.position, Point::new(10, 40));
                assert_eq!(view.bounds.size, Size::new(180, 180 - 30));
            }
        }
    }

    #[test]
    fn test_complex_tabbed_panels() {
        let mut scene = Scene::new();
        // tab panel test
        let tabbed_panel: ViewId = "tabbed_panel_id".into();
        let tabs: ViewId = "tabs".into();
        {
            let mut tabbed_panel_view: View = View {
                name: tabbed_panel.clone(),
                .. Default::default()
            };
            tabbed_panel_view.h_flex = Flex::Resize;
            tabbed_panel_view.v_flex = Flex::Resize;
            tabbed_panel_view.layout = Some(layout_tabbed_panel);
            scene.add_view_to_parent(tabbed_panel_view, &scene.root_id());

            // tab panel tabs has intrisic height but flex width
            let mut tabbed_panel_tabs: View = View {
                name: tabs.clone(),
                .. Default::default()
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
            let mut view = make_standard_view(&tab1);
            view.h_flex = Flex::Resize;
            view.v_flex = Flex::Resize;
            view.layout = Some(layout_hbox_2);
            scene.add_view_to_parent(view, &tabbed_panel);

            let b1: ViewId = "tab1_button1".into();
            {
                let mut button = make_standard_view(&b1);
                button.title = "abc".into();
                button.v_align = Start;
                button.layout = Some(layout_button);
                scene.add_view_to_parent(button, &tab1);
            }

            let b2: ViewId = "tab1_button2".into();
            {
                let mut button = make_standard_view(&b2);
                button.v_align = Center;
                button.layout = Some(layout_button);
                scene.add_view_to_parent(button, &tab1);
            }

            let b3: ViewId = "tab1_button3".into();
            {
                let mut button = make_standard_view(&b3);
                button.v_align = End;
                button.layout = Some(layout_button);
                scene.add_view_to_parent(button, &tab1);
            }
        }

        // second tab panel is a vbox with three buttons horizontally centered
        // and the first one takes all vertical space and horizontal space
        let tab2: ViewId = "tab2".into();
        {
            let mut view = make_standard_view(&tab2);
            view.h_flex = Flex::Resize;
            view.v_flex = Flex::Resize;
            view.layout = Some(layout_vbox);
            scene.add_view_to_parent(view, &tabbed_panel);

            let b1: ViewId = "tab2_button1".into();
            {
                let mut b1_view = make_standard_view(&b1);
                b1_view.h_align = Start;
                b1_view.h_flex = Flex::Resize;
                b1_view.v_flex = Flex::Resize;
                b1_view.title = "b11".into();
                b1_view.layout = Some(layout_std_panel);
                scene.add_view_to_parent(b1_view, &tab2);
            }

            let b2: ViewId = "tab2_button2".into();
            {
                let mut b2_view = make_standard_view(&b2);
                b2_view.h_align = End;
                b2_view.h_flex = Flex::Intrinsic;
                b2_view.v_flex = Flex::Intrinsic;
                b2_view.title = "b11".into();
                b2_view.layout = Some(layout_button);
                scene.add_view_to_parent(b2_view, &tab2);
            }
        }

        // third tab panel lets children be absolutely positioned and sizes self to the center with a fixed width and height of 100
        let tab3: ViewId = "tab3".into();
        {
            let mut view = make_standard_view(&tab3);
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
            scene.view_bounds(&scene.root_id()),
            Bounds::new(0, 0, 200, 200)
        );
        assert_eq!(
            scene.view_bounds(&tabbed_panel),
            Bounds::new(0, 0, 200, 200)
        );
        assert_eq!(scene.view_bounds(&tabs), Bounds::new(0, 0, 200, 10));
        assert_eq!(scene.view_bounds(&tab1), Bounds::new(0, 10, 200, 190));
        assert_eq!(scene.view_bounds(&tab2), Bounds::new(0, 10, 200, 190));
        assert_eq!(scene.view_bounds(&tab3), Bounds::new(0, 10, 200, 190));

        assert_eq!(
            scene.view_bounds(&"tab1_button1".into()),
            Bounds::new(0, 0, 30, 10)
        );
        assert_eq!(
            scene.view_bounds(&"tab2_button1".into()),
            Bounds::new(0, 0, 200, 180)
        );
        assert_eq!(
            scene.view_bounds(&"tab2_button2".into()),
            Bounds::new(170, 180, 30, 10)
        );
    }

    fn make_standard_view(name: &ViewId) -> View {
        View {
            name: name.clone(),
            .. Default::default()
        }
    }
}
