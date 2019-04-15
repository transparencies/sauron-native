use crate::WidgetNode;
use sauron_vdom::builder::element;
use sauron_vdom::builder::Attribute;

#[derive(Debug, Clone)]
pub enum Widget {
    Column,
    Row,
    Button(String),
    Text(String),
}

pub fn widget<'a, A, C>(widget: Widget, attrs: A, children: C) -> WidgetNode
where
    C: AsRef<[WidgetNode]>,
    A: AsRef<[Attribute<'a>]>,
{
    element(widget, attrs, children)
}

pub fn column<'a, A, C>(attrs: A, children: C) -> WidgetNode
where
    C: AsRef<[WidgetNode]>,
    A: AsRef<[Attribute<'a>]>,
{
    widget(Widget::Column, attrs, children)
}

pub fn row<'a, A, C>(attrs: A, children: C) -> WidgetNode
where
    C: AsRef<[WidgetNode]>,
    A: AsRef<[Attribute<'a>]>,
{
    widget(Widget::Row, attrs, children)
}

pub fn button<'a, A>(attrs: A, txt: &str) -> WidgetNode
where
    A: AsRef<[Attribute<'a>]>,
{
    widget(Widget::Button(txt.to_string()), attrs, [])
}

pub fn text(txt: &str) -> WidgetNode {
    widget(Widget::Text(txt.to_string()), [], [])
}