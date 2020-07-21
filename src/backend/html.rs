//! html backend where all the functionalities is offloaded into sauron
use crate::{
    util, widget::attribute::find_value, AttribKey, Attribute, Backend, Component, Widget,
};
use sauron::{
    html::{attributes::*, div, img, input, text},
    prelude::*,
};
use std::{fmt::Debug, marker::PhantomData};

mod convert_event;

/// holds the user application
pub struct HtmlApp<APP, MSG>
where
    MSG: Clone + Debug + 'static,
    APP: Component<MSG> + 'static,
{
    app: APP,
    _phantom_data: PhantomData<MSG>,
}

/// html backend
pub struct HtmlBackend<APP, MSG>
where
    MSG: Clone + Debug + 'static,
    APP: Component<MSG> + 'static,
{
    _phantom_app: PhantomData<(APP, MSG)>,
}

impl<APP, MSG> HtmlApp<APP, MSG>
where
    MSG: Clone + Debug + 'static,
    APP: Component<MSG> + 'static,
{
    fn new(app: APP) -> Self {
        HtmlApp {
            app,
            _phantom_data: PhantomData,
        }
    }
}

impl<APP, MSG> sauron::Component<MSG> for HtmlApp<APP, MSG>
where
    MSG: Clone + Debug + 'static,
    APP: Component<MSG> + 'static,
{
    fn update(&mut self, msg: MSG) -> sauron::dom::cmd::Cmd<sauron::Program<Self, MSG>, MSG> {
        self.app.update(msg);
        sauron::dom::cmd::Cmd::none()
    }

    fn view(&self) -> sauron::Node<MSG> {
        let view = self.app.view();
        let html_view = widget_tree_to_html_node(view, &mut 0);
        html_view
    }
}

impl<APP, MSG> Backend<APP, MSG> for HtmlBackend<APP, MSG>
where
    MSG: Clone + Debug + 'static,
    APP: Component<MSG> + 'static,
{
    fn init(app: APP) -> Self {
        log::trace!("Html app started..");
        let html_app = HtmlApp::new(app);
        sauron::Program::mount_to_body(html_app);
        HtmlBackend {
            _phantom_app: PhantomData,
        }
    }
}

/// converts widget virtual node tree into an html node tree
pub fn widget_tree_to_html_node<MSG>(
    widget_node: crate::Node<MSG>,
    cur_node_idx: &mut usize,
) -> sauron::Node<MSG>
where
    MSG: Clone + Debug + 'static,
{
    match widget_node {
        crate::Node::Element(widget) => {
            widget_to_html(widget.tag, widget.attrs, widget.children, cur_node_idx)
        }
        crate::Node::Text(txt) => {
            *cur_node_idx += 1;
            text(txt)
        }
    }
}

/// convert Widget into an equivalent html node
fn widget_to_html<MSG>(
    widget: Widget,
    attrs: Vec<Attribute<MSG>>,
    widget_children: Vec<crate::Node<MSG>>,
    cur_node_idx: &mut usize,
) -> sauron::Node<MSG>
where
    MSG: Clone + Debug + 'static,
{
    let mut html_children = vec![];
    for widget_child in widget_children {
        *cur_node_idx += 1;
        // convert all widget child to an html child node
        let html_child: sauron::Node<MSG> = widget_tree_to_html_node(widget_child, cur_node_idx);
        html_children.push(html_child);
    }
    match widget {
        Widget::Vbox => div(
            vec![styles(vec![
                ("display", "flex"),
                ("flex-direction", "column"),
            ])],
            html_children,
        ),
        Widget::Hbox => div(
            vec![styles(vec![("display", "flex"), ("flex-direction", "row")])],
            html_children,
        ),
        //TODO: vpane and hpane should be draggable
        Widget::Vpane => div(
            vec![
                styles(vec![("display", "flex"), ("flex-direction", "column")]),
                width("100%"),
                height("100%"),
            ],
            html_children,
        ),
        // hpane will split the 2 children with 50-50 width
        // and a 100% height
        Widget::Hpane => {
            html_children.iter_mut().for_each(|child| {
                child.add_attributes_ref_mut(vec![styles([("width", "50%"), ("height", "100%")])])
            });
            div(
                vec![styles(vec![
                    ("display", "flex"),
                    ("flex-direction", "row"),
                    //("width", "100%"),
                    //("height", "100%"),
                ])],
                html_children,
            )
        }
        // the children in overlay will be all in absolute
        Widget::Overlay => {
            html_children.iter_mut().for_each(|child| {
                child.add_attributes_ref_mut(vec![styles([("position", "absolute")])]);
            });
            div(vec![class("overlay")], html_children)
        }
        Widget::GroupBox => div(vec![], html_children),
        Widget::Label => {
            let value = find_value(AttribKey::Value, &attrs)
                .map(|v| v.to_string())
                .unwrap_or(String::new());
            label(vec![styles([("user-select", "none")])], vec![text(value)])
        }
        Widget::Button => {
            let label = find_value(AttribKey::Label, &attrs)
                .map(|v| v.to_string())
                .unwrap_or(String::new());

            let svg_image_data = find_value(AttribKey::SvgImage, &attrs)
                .map(|v| v.as_bytes().map(|v| v.to_vec()))
                .flatten();

            let mut attributes = vec![];
            for att in attrs {
                match att.name() {
                    AttribKey::ClickEvent => {
                        for cb in att.get_callback() {
                            let cb = cb.clone();
                            attributes.push(on_click(move |ev| {
                                cb.emit(convert_event::from_mouse_event(ev))
                            }))
                        }
                    }
                    _ => (),
                }
            }

            button(
                vec![],
                vec![
                    text(label),
                    if let Some(svg_image_data) = svg_image_data {
                        img(
                            vec![
                                styles([
                                    ("width", "100%"),
                                    ("height", "auto"),
                                    ("max-width", "800px"),
                                ]),
                                src(format!(
                                    "data:image/svg+xml;base64,{}",
                                    base64::encode(&svg_image_data)
                                )),
                            ],
                            vec![],
                        )
                    } else {
                        text("")
                    },
                ],
            )
            .add_attributes(attributes)
        }
        Widget::Paragraph => {
            let txt_value = find_value(AttribKey::Value, &attrs)
                .map(|v| v.to_string())
                .unwrap_or(String::new());
            p(vec![], vec![text(txt_value)])
        }
        Widget::TextInput => {
            let txt_value = find_value(AttribKey::Value, &attrs)
                .map(|v| v.to_string())
                .unwrap_or(String::new());
            let mut attributes = vec![];
            for att in attrs {
                match att.name() {
                    AttribKey::InputEvent => {
                        for cb in att.get_callback() {
                            let cb = cb.clone();
                            attributes.push(on_input(move |ev| {
                                cb.emit(convert_event::to_input_event(ev))
                            }));
                        }
                    }
                    _ => (),
                }
            }
            input(vec![r#type("text"), value(txt_value)], vec![]).add_attributes(attributes)
        }
        Widget::TextArea => {
            let txt_value = find_value(AttribKey::Value, &attrs)
                .map(|v| v.to_string())
                .unwrap_or(String::new());
            let attributes = attrs
                .into_iter()
                .filter_map(|att| match att.name() {
                    /*
                    AttribKey::InputEvent => att
                        .take_callback()
                        .map(|cb| on_input(move |ev| cb.emit(ev))),
                    */
                    _ => None,
                })
                .collect();
            textarea(
                vec![value(&txt_value), styles([("height", "100%")])],
                vec![text(txt_value)],
            )
            .add_attributes(attributes)
        }
        Widget::Checkbox => {
            let cb_label = find_value(AttribKey::Label, &attrs)
                .map(|v| v.to_string())
                .unwrap_or(String::new());
            let cb_value = find_value(AttribKey::Value, &attrs)
                .map(|v| v.as_bool())
                .unwrap_or(false);
            let checked = attrs_flag([("checked", "checked", cb_value)]);
            let widget_id = format!("checkbox_{}", cur_node_idx);

            div(
                vec![],
                vec![
                    input(vec![type_("checkbox"), id(&widget_id)], vec![]).add_attributes(checked),
                    label(vec![for_(&widget_id)], vec![text(cb_label)]),
                ],
            )
        }
        Widget::Radio => {
            let cb_label = find_value(AttribKey::Label, &attrs)
                .map(|v| v.to_string())
                .unwrap_or(String::new());
            let cb_value = find_value(AttribKey::Value, &attrs)
                .map(|v| v.as_bool())
                .unwrap_or(false);
            let checked = attrs_flag([("checked", "checked", cb_value)]);
            let widget_id = format!("radio_{}", cur_node_idx);
            div(
                vec![],
                vec![
                    input(vec![type_("radio"), id(&widget_id)], vec![]).add_attributes(checked),
                    label(vec![for_(&widget_id)], vec![text(cb_label)]),
                ],
            )
        }
        Widget::Image => {
            let empty = vec![];
            let bytes = find_value(AttribKey::Data, &attrs)
                .map(|v| v.as_bytes())
                .flatten()
                .unwrap_or(&empty);

            let mime_type = util::image_mime_type(bytes).expect("unsupported image");
            div(
                vec![],
                vec![img(
                    vec![src(format!(
                        "data:{};base64,{}",
                        mime_type,
                        base64::encode(bytes)
                    ))],
                    vec![],
                )],
            )
        }
        Widget::Svg => {
            let empty = vec![];
            let bytes = find_value(AttribKey::Data, &attrs)
                .map(|v| v.as_bytes())
                .flatten()
                .unwrap_or(&empty);
            div(
                vec![],
                vec![img(
                    vec![src(format!(
                        "data:image/svg+xml;base64,{}",
                        base64::encode(bytes)
                    ))],
                    vec![],
                )],
            )
        }
    }
}
