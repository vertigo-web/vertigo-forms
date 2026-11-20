use std::rc::Rc;

use vertigo::{AttrGroup, Computed, DomNode, Value, css, dom};
use vertigo_forms::{
    DragGroup, DragGroups, DragGroupsParams, DragMove, OnDragMove, RenderDragItem,
    RenderGroupHeader, RenderItemAside,
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

    // A pin at the end of a chore, shown with the pointer over it or the keyboard on it, and
    // always on a pinned one
    let pinned = Value::new(Vec::<String>::new());
    let render_item_aside: RenderItemAside<String> = Rc::new({
        let pinned = pinned.clone();
        move |item: &String, shown: Computed<bool>| {
            let is_pinned = pinned.map({
                let item = item.clone();
                move |pinned| pinned.contains(&item)
            });
            let visible = Computed::from({
                let is_pinned = is_pinned.clone();
                move |ctx| shown.get(ctx) || is_pinned.get(ctx)
            });
            let pin_css = visible.map(|visible| {
                let opacity = if visible { "1" } else { "0" };
                css! {"
                    position: absolute;
                    right: 0;
                    padding: 0 2px;
                    border: none;
                    background: white;
                    opacity: {opacity};

                    :focus-visible {
                        opacity: 1;
                    };
                "}
            });
            let toggle = {
                let (pinned, item) = (pinned.clone(), item.clone());
                move |_| {
                    pinned.change(|pinned| match pinned.iter().position(|it| *it == item) {
                        Some(index) => {
                            pinned.remove(index);
                        }
                        None => pinned.push(item.clone()),
                    })
                }
            };
            Some(dom! {
                <button
                    type="button"
                    css={pin_css}
                    aria-label={format!("Pin {item}")}
                    aria-pressed={is_pinned.map(|pinned| pinned.to_string())}
                    on_click={toggle}
                >
                    "📌"
                </button>
            })
        }
    });
    let pinned_list = pinned.map(|pinned| match pinned.is_empty() {
        true => "Pinned: nothing".to_string(),
        false => format!("Pinned: {}", pinned.join(", ")),
    });

    let params = DragGroupsParams {
        render_item_aside,
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
        <p>"Drag a chore to another group, or pick it and press Here. The pin at its end pins it."</p>
        <DragGroups
            groups={groups}
            {render_header}
            {render_item}
            {on_move}
            {params}
        />
        <p>"Groups: " {summary}</p>
        <p>{pinned_list}</p>
    }
}
