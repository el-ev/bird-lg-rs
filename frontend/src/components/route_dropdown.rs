use common::models::NodeProtocol;
use wasm_bindgen::JsCast;
use web_sys::{Element, MouseEvent};
use yew::prelude::*;
use yew_router::prelude::*;

use crate::{routes::Route, store::LgStateHandle, utils::is_dn42_domain};

const ROUTE_DROPDOWN_MENU_ID: &str = "lg-route-path-menu";

struct RouteMenuItem {
    label: String,
    path: String,
    route: Option<Route>,
    children: Vec<RouteMenuChild>,
}

struct RouteMenuChild {
    label: String,
    path: String,
    route: Route,
}

fn build_route_menu_items(nodes: &[NodeProtocol]) -> Vec<RouteMenuItem> {
    let mut static_routes: Vec<Route> = vec![Route::Protocols, Route::Peering, Route::WireGuard];

    if is_dn42_domain() {
        static_routes.push(Route::Dn42);
    }

    let mut items: Vec<RouteMenuItem> = static_routes
        .iter()
        .map(|route| {
            let path = route.to_path();
            RouteMenuItem {
                label: path.clone(),
                path,
                route: Some(route.clone()),
                children: Vec::new(),
            }
        })
        .collect();

    let node_children: Vec<RouteMenuChild> = nodes
        .iter()
        .map(|node| {
            let route = Route::Node {
                name: node.name.clone(),
            };
            RouteMenuChild {
                label: node.name.clone(),
                path: route.to_path(),
                route,
            }
        })
        .collect();

    let insert_pos = items.len().saturating_sub(1);
    items.insert(
        insert_pos,
        RouteMenuItem {
            label: String::from("/node"),
            path: String::from("/node"),
            route: None,
            children: node_children,
        },
    );

    items
}

#[derive(Properties, PartialEq)]
pub struct RouteDropdownProps {
    pub current_path: AttrValue,
    pub current_node: Option<String>,
}

#[function_component(RouteDropdown)]
pub fn route_dropdown(props: &RouteDropdownProps) -> Html {
    let state = use_context::<LgStateHandle>().expect("no app state found");
    let menu_open = use_state(|| false);
    let container_ref = use_node_ref();

    let open_menu_mouse = {
        let menu_open = menu_open.clone();
        Callback::from(move |_| {
            if !*menu_open {
                menu_open.set(true);
            }
        })
    };

    let close_menu_mouse = {
        let menu_open = menu_open.clone();
        let container_ref = container_ref.clone();
        Callback::from(move |event: MouseEvent| {
            let stays_inside = container_ref
                .cast::<Element>()
                .and_then(|container| {
                    event
                        .related_target()
                        .and_then(|related| related.dyn_into::<Element>().ok())
                        .map(|target| container.contains(Some(&target)))
                })
                .unwrap_or(false);

            if !stays_inside && *menu_open {
                menu_open.set(false);
            }
        })
    };

    let open_menu_focus = {
        let menu_open = menu_open.clone();
        Callback::from(move |_| {
            if !*menu_open {
                menu_open.set(true);
            }
        })
    };

    let close_menu_focus = {
        let menu_open = menu_open.clone();
        let container_ref = container_ref.clone();
        Callback::from(move |event: FocusEvent| {
            let stays_inside = container_ref
                .cast::<Element>()
                .and_then(|container| {
                    event
                        .related_target()
                        .and_then(|related| related.dyn_into::<Element>().ok())
                        .map(|target| container.contains(Some(&target)))
                })
                .unwrap_or(false);

            if !stays_inside && *menu_open {
                menu_open.set(false);
            }
        })
    };

    let open_menu_click = {
        let menu_open = menu_open.clone();
        Callback::from(move |_| {
            if !*menu_open {
                menu_open.set(true);
            }
        })
    };

    let items = build_route_menu_items(&state.nodes);
    let aria_expanded = if *menu_open { "true" } else { "false" };

    html! {
        <span
            class={classes!(
                "route-dropdown",
                if *menu_open { Some("route-dropdown--open") } else { None },
            )}
            ref={container_ref}
            onmouseenter={open_menu_mouse}
            onmouseleave={close_menu_mouse}
            onfocusin={open_menu_focus}
            onfocusout={close_menu_focus}
        >
            <button
                type="button"
                class="route-dropdown__toggle"
                aria-haspopup="menu"
                aria-expanded={aria_expanded}
                aria-controls={ROUTE_DROPDOWN_MENU_ID}
                onclick={open_menu_click}
            >
                <span>{ props.current_path.clone() }</span>
            </button>
            <div class="route-dropdown__menu" id={ROUTE_DROPDOWN_MENU_ID} role="menu">
                <ul class="route-dropdown__list">
                    {
                        for items.into_iter().map(|item| {
                            let mut item_classes = classes!("route-dropdown__item");
                            let child_match = if let Some(node) = props.current_node.as_ref() {
                                item
                                    .children
                                    .iter()
                                    .any(|child| &child.label == node)
                            } else {
                                false
                            };
                            if props.current_path.as_ref() == item.path || child_match {
                                item_classes.push("is-active");
                            }
                            if !item.children.is_empty() {
                                item_classes.push("has-children");
                            }

                            let item_onclick = item.route.as_ref().map(|_| {
                                let menu_open = menu_open.clone();
                                Callback::from(move |_| {
                                    menu_open.set(false);
                                })
                            });

                            let has_children = !item.children.is_empty();
                            let child_menu = if !has_children {
                                html! {}
                            } else {
                                let cols = if item.children.len() > 8 { 2 } else { 1 };
                                let style = format!("--cols: {}", cols);
                                html! {
                                    <ul class="route-dropdown__sub" style={style}>
                                        {
                                            for item.children.into_iter().map(|child| {
                                                let RouteMenuChild { label, path, route } = child;
                                                let mut child_classes = classes!("route-dropdown__sub-item");
                                                let node_selected = if let Some(node) = props.current_node.as_ref() {
                                                    node == &label
                                                } else {
                                                    false
                                                };
                                                let is_child_active = node_selected
                                                    || props.current_path.as_ref() == path;
                                                if is_child_active {
                                                    child_classes.push("is-active");
                                                }
                                                let on_child_click = {
                                                    let menu_open = menu_open.clone();
                                                    Callback::from(move |_| {
                                                        menu_open.set(false);
                                                    })
                                                };
                                                html! {
                                                    <li class={child_classes} onclick={on_child_click}>
                                                        <Link<Route> to={route} classes="route-dropdown__sub-link">
                                                            { label }
                                                        </Link<Route>>
                                                    </li>
                                                }
                                            })
                                        }
                                    </ul>
                                }
                            };

                            html! {
                                <li class={item_classes} onclick={item_onclick} onmouseenter={
                                    if has_children {
                                        Some(Callback::from(move |e: MouseEvent| {
                                            if let Some(target) = e.target_dyn_into::<Element>() {
                                                let rect = target.get_bounding_client_rect();
                                                    let window = web_sys::window().unwrap();
                                                    let window_width = window.inner_width().unwrap().as_f64().unwrap();
                                                    if rect.right() + 180.0 > window_width {
                                                        if let Ok(sub) = target.query_selector(".route-dropdown__sub")
                                                            && let Some(sub) = sub {
                                                                let _ = sub.class_list().add_1("opens-left");
                                                            }
                                                    } else if let Ok(sub) = target.query_selector(".route-dropdown__sub")
                                                        && let Some(sub) = sub {
                                                            let _ = sub.class_list().remove_1("opens-left");
                                                        }
                                            }
                                        }))
                                    } else {
                                        None
                                    }
                                }>
                                    {
                                        if let Some(route) = item.route {
                                            html! {
                                                <Link<Route> to={route} classes="route-dropdown__link">
                                                    { item.label }
                                                </Link<Route>>
                                            }
                                        } else {
                                            html! {
                                                <span
                                                    class="route-dropdown__link route-dropdown__label"
                                                    tabindex="0"
                                                    role="button"
                                                    aria-haspopup="true"
                                                >
                                                    { item.label.to_owned() + "/" }
                                                </span>
                                            }
                                        }
                                    }
                                    { child_menu }
                                </li>
                            }
                        })
                    }
                </ul>
            </div>
        </span>
    }
}
