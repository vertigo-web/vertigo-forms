use std::rc::Rc;

use vertigo::{AttrGroup, DomNode, Value, dom};
use vertigo_forms::{
    DragGroup, DragGroups, DragGroupsParams, DragMove, OnDragMove, RenderDragItem,
    RenderGroupHeader,
};

pub fn drag_groups() -> DomNode {
    let groups = Value::new(vec![
        DragGroup {
            key: "To do",
            items: ["Wash", "Cook", "Sweep"].map(String::from).to_vec(),
        },
        DragGroup {
            key: "Doing",
            items: vec!["Shop".to_string()],
        },
        DragGroup {
            key: "Done",
            items: vec![],
        },
    ]);

    // Moved at once; an app may wait for a server instead
    let on_move: OnDragMove<&'static str, String> = Rc::new({
        let groups = groups.clone();
        move |moved: DragMove<&'static str, String>| {
            groups.change(|groups| {
                for group in groups.iter_mut() {
                    group.items.retain(|item| *item != moved.item);
                }
                if let Some(group) = groups.iter_mut().find(|group| group.key == moved.to) {
                    group.items.push(moved.item);
                }
            });
        }
    });

    let summary = groups.map(|groups| {
        groups
            .iter()
            .map(|group| match group.items.is_empty() {
                true => format!("{}: nothing", group.key),
                false => format!("{}: {}", group.key, group.items.join(", ")),
            })
            .collect::<Vec<_>>()
            .join("; ")
    });

    let params = DragGroupsParams {
        empty_label: Some("Nothing".to_string()),
        group_attrs: Rc::new(|key: &&'static str| {
            AttrGroup::from([
                ("role".to_string(), "group".into()),
                ("aria-label".to_string(), (*key).into()),
            ])
        }),
        render_footer: Rc::new(|key: &&'static str| {
            (*key == "Done").then(|| dom! { <small>"Well done!"</small> })
        }),
        ..Default::default()
    };

    let render_header: RenderGroupHeader<&'static str> =
        Rc::new(|key| dom! { <strong>{*key}</strong> });
    let render_item: RenderDragItem<String> = Rc::new(|item| dom! { <span>{item}</span> });

    dom! {
        <p>"Drag a chore to another group, or pick it and press Here."</p>
        <DragGroups
            groups={groups}
            {render_header}
            {render_item}
            {on_move}
            {params}
        />
        <p>"Groups: " {summary}</p>
    }
}
