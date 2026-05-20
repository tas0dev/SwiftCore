//! Simple SwiftUI-like DSL builders and macros for viewKit
//! Provides builder structs and macro_rules! wrappers such as `card!()`, `text!()` and `button!()`

use std::rc::Rc;
use std::fmt;

#[derive(Clone)]
pub struct OnClick(Rc<dyn Fn()>);

impl OnClick {
    pub fn new<F: 'static + Fn()>(f: F) -> Self { Self(Rc::new(f)) }
}

impl fmt::Debug for OnClick {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("OnClick(...)")
    }
}

#[derive(Debug, Clone)]
pub enum UIElement {
    Card(Card),
    Text(Text),
    Button(Button),
    Bundle(Vec<UIElement>),
}

#[derive(Debug, Clone, Default)]
pub struct Card {
    pub children: Vec<UIElement>,
    pub color: Option<String>,
}

impl Card {
    pub fn new() -> Self { Self::default() }
    pub fn children(mut self, elems: impl IntoIterator<Item=UIElement>) -> Self {
        self.children.extend(elems);
        self
    }
    pub fn color<T: Into<String>>(mut self, c: T) -> Self {
        self.color = Some(c.into());
        self
    }
    pub fn into_elem(self) -> UIElement { UIElement::Card(self) }
}

 #[derive(Debug, Clone, Default)]
 pub struct Text {
     pub text: String,
     pub color: Option<String>,
 }

impl Text {
    pub fn new<T: Into<String>>(s: T) -> Self { Self { text: s.into(), color: None } }
    pub fn color<T: Into<String>>(mut self, c: T) -> Self { self.color = Some(c.into()); self }
    pub fn into_elem(self) -> UIElement { UIElement::Text(self) }
}

 #[derive(Debug, Clone, Default)]
 pub struct Button {
     pub label: String,
     pub on_click: Option<OnClick>,
 }

impl Button {
    pub fn new<T: Into<String>>(label: T) -> Self { Self { label: label.into(), on_click: None } }
    pub fn on_click<F: 'static + Fn()>(mut self, f: F) -> Self { self.on_click = Some(OnClick::new(f)); self }
    pub fn into_elem(self) -> UIElement { UIElement::Button(self) }
}

 // convenience conversions
 impl From<Card> for UIElement { fn from(c: Card) -> Self { UIElement::Card(c) } }
 impl From<Text> for UIElement { fn from(t: Text) -> Self { UIElement::Text(t) } }
 impl From<Button> for UIElement { fn from(b: Button) -> Self { UIElement::Button(b) } }

 // Macros
 #[macro_export]
 macro_rules! card {
     () => {
         crate::ui::Card::new()
     };
 }

 #[macro_export]
 macro_rules! text {
     ($s:expr) => {
         crate::ui::Text::new($s)
     };
 }

 #[macro_export]
 macro_rules! button {
     ($s:expr) => {
         crate::ui::Button::new($s)
     };
 }

 #[macro_export]
 macro_rules! bundle {
     ( [ $( $e:expr ),* $(,)? ] ) => {
         crate::ui::UIElement::Bundle(vec![ $( $e.into() ),* ])
     };
 }


