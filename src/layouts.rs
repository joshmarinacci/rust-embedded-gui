use log::info;
use crate::geom::{Insets, Size};
use crate::LayoutEvent;
use crate::view::Align::{Center, End, Start};
use crate::view::{Flex};

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
        .get_children_ids_filtered(&pass.target, |v| v.v_flex == Flex::Intrinsic);
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

pub fn layout_hbox(pass: &mut LayoutEvent) {
    let Some(parent) = pass.scene.get_view_mut(&pass.target) else {
        return;
    };
    // layout self
    parent.bounds.size = pass.space.clone();
    let padding = parent.padding.clone();
    let available_space: Size = pass.space - padding;

    // get the fixed children
    let fixed_kids = pass
        .scene
        .get_children_ids_filtered(&pass.target, |v| v.h_flex == Flex::Intrinsic);

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
    let avail_horizontal_space = (pass.space - padding).h - kids_sum;

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
    let avail_h = pass.space.h - padding.top - padding.bottom;
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
