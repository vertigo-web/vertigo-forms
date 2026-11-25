use std::rc::Rc;

use vertigo::{AttrGroup, Computed, DomNode, Value, css, dom, transaction};
use vertigo_forms::{
    DragGroup, DragGroups, DragGroupsParams, DragMove, OnDragMove, RenderDragItem,
    RenderGroupHeader, RenderItemAside,
};

/// How wide the pin over the end of a chore is, and the room the chore keeps for it.
const PIN_WIDTH: &str = "22px";
const PIN_ROOM: &str = "26px";

/// Pinned chores stand first in their group, in the order they had.
fn pinned_first(groups: &mut [DragGroup<&'static str, String>], pinned: &[String]) {
    for group in groups {
        group.items.sort_by_key(|item| !pinned.contains(item));
    }
}

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

    let pinned = Value::new(Vec::<String>::new());

    // Moved at once; an app may wait for a server instead
    let on_move: OnDragMove<&'static str, String> = Rc::new({
        let (groups, pinned) = (groups.clone(), pinned.clone());
        move |moved: DragMove<&'static str, String>| {
            let pinned = transaction(|ctx| pinned.get(ctx));
            groups.change(|groups| {
                for group in groups.iter_mut() {
                    group.items.retain(|item| *item != moved.item);
                }
                if let Some(group) = groups.iter_mut().find(|group| group.key == moved.to) {
                    group.items.push(moved.item);
                }
                pinned_first(groups, &pinned);
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

    // A pin over the end of a chore, which puts it first in its group: shown with the pointer over
    // the chore, the keyboard on it or the chore picked, and always on a pinned one - which a tap
    // reaches too
    let render_item_aside: RenderItemAside<String> = Rc::new({
        let (groups, pinned) = (groups.clone(), pinned.clone());
        move |item: &String, shown: Computed<bool>| {
            let is_pinned = pinned.map({
                let item = item.clone();
                move |pinned| pinned.contains(&item)
            });
            // Shown, it takes the pointer when the component lets it; pinned, always
            let pin_css = Computed::from({
                let is_pinned = is_pinned.clone();
                move |ctx| (shown.get(ctx), is_pinned.get(ctx))
            })
            .map(|(shown, is_pinned)| {
                let opacity = if shown || is_pinned { "1" } else { "0" };
                let pointer_events = if is_pinned { "auto" } else { "inherit" };
                // In the room the chore keeps for it at its end
                css! {"
                    position: absolute;
                    top: 0;
                    right: 0;
                    bottom: 0;
                    width: {PIN_WIDTH};
                    padding: 0;
                    border: none;
                    background: none;
                    opacity: {opacity};
                    pointer-events: {pointer_events};
                "}
            });
            let toggle = {
                let (groups, pinned, item) = (groups.clone(), pinned.clone(), item.clone());
                move |_| {
                    let mut now = transaction(|ctx| pinned.get(ctx));
                    match now.iter().position(|it| *it == item) {
                        Some(index) => {
                            now.remove(index);
                        }
                        None => now.push(item.clone()),
                    }
                    groups.change(|groups| pinned_first(groups, &now));
                    pinned.set(now);
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

    let defaults = DragGroupsParams::default();
    let params = DragGroupsParams {
        render_item_aside,
        // Room for the pin at the end of a chore, which doesn't cover its name then
        item_css: defaults.item_css.clone() + css! {"padding-right: {PIN_ROOM};"},
        empty_label: Some("Nothing".to_string()),
        group_attrs: Rc::new(|key: &&'static str| {
            AttrGroup::from([
                ("role".to_string(), "group".into()),
                ("aria-label".to_string(), (*key).into()),
            ])
        }),
        // A chore added renders the items of "To do" anew, not the button below them, which keeps
        // the focus
        render_footer: Rc::new({
            let (groups, added) = (groups.clone(), Value::new(0));
            move |key: &&'static str| match *key {
                "To do" => {
                    let add = {
                        let (groups, added) = (groups.clone(), added.clone());
                        move |_| {
                            let number = transaction(|ctx| added.get(ctx)) + 1;
                            added.set(number);
                            groups.change(|groups| {
                                if let Some(group) = groups.iter_mut().find(|g| g.key == "To do") {
                                    group.items.push(format!("Chore {number}"));
                                }
                            });
                        }
                    };
                    Some(dom! { <button type="button" on_click={add}>"Add a chore"</button> })
                }
                "Done" => Some(dom! { <small>"Well done!"</small> }),
                _ => None,
            }
        }),
        ..defaults
    };

    let render_header: RenderGroupHeader<&'static str> =
        Rc::new(|key| dom! { <strong>{*key}</strong> });
    let render_item: RenderDragItem<String> = Rc::new(|item| dom! { <span>{item}</span> });

    dom! {
        <p>"Drag a chore to another group, or pick it and press Here. The pin at its end puts it first."</p>
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
