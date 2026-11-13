//! `html.parse(text)` → nút gốc; mỗi nút có `select`, `selectFirst`, `text`, `html`, `outerHtml`, `attr`,
//! `children`. Cây HTML giữ ở Rust (scraper), nút JS chỉ trỏ vào — bộ dọn rác JS giải phóng khi hết dùng.

use super::HostState;
use ego_tree::NodeId;
use rquickjs::class::Trace;
use rquickjs::{Class, Ctx, Exception, Function, IntoJs, JsLifetime, Result, Value};
use scraper::{ElementRef, Html, Selector};
use std::rc::Rc;

pub(crate) const JS: &str = r#"
  define("html", freeze({ parse: (text) => host.html_parse(String(text == null ? "" : text)) }));
"#;

#[derive(Trace, JsLifetime)]
#[rquickjs::class]
pub(crate) struct HtmlNode {
    #[qjs(skip_trace)]
    doc: Rc<Html>,
    #[qjs(skip_trace)]
    id: NodeId,
}

fn selector(ctx: &Ctx<'_>, css: &str) -> Result<Selector> {
    Selector::parse(css).map_err(|e| Exception::throw_message(ctx, &format!("css không hợp lệ \"{css}\": {e}")))
}

/// Gộp khoảng trắng liên tiếp thành một dấu cách, bỏ đầu cuối.
fn squash(text: &str) -> String {
    text.split_whitespace().collect::<Vec<_>>().join(" ")
}

impl HtmlNode {
    fn element(&self) -> Option<ElementRef<'_>> {
        self.doc.tree.get(self.id).and_then(ElementRef::wrap)
    }

    /// Nút gốc của tài liệu không phải phần tử — thao tác trên `<html>`.
    fn element_or_root(&self) -> ElementRef<'_> {
        self.element().unwrap_or_else(|| self.doc.root_element())
    }

    fn wrap(&self, element: ElementRef<'_>) -> HtmlNode {
        HtmlNode { doc: self.doc.clone(), id: element.id() }
    }
}

#[rquickjs::methods]
impl HtmlNode {
    pub fn select<'js>(&self, ctx: Ctx<'js>, css: String) -> Result<Vec<HtmlNode>> {
        let selector = selector(&ctx, &css)?;
        Ok(match self.element() {
            Some(element) => element.select(&selector).map(|e| self.wrap(e)).collect(),
            None => self.doc.select(&selector).map(|e| self.wrap(e)).collect(),
        })
    }

    /// Không có thì trả `null` (không phải `undefined`).
    #[qjs(rename = "selectFirst")]
    pub fn select_first<'js>(&self, ctx: Ctx<'js>, css: String) -> Result<Value<'js>> {
        let selector = selector(&ctx, &css)?;
        let found = match self.element() {
            Some(element) => element.select(&selector).next().map(|e| self.wrap(e)),
            None => self.doc.select(&selector).next().map(|e| self.wrap(e)),
        };
        match found {
            Some(node) => Ok(Class::instance(ctx, node)?.into_value()),
            None => Ok(Value::new_null(ctx)),
        }
    }

    pub fn text(&self) -> String {
        squash(&self.element_or_root().text().collect::<String>())
    }

    pub fn html(&self) -> String {
        self.element_or_root().inner_html()
    }

    #[qjs(rename = "outerHtml")]
    pub fn outer_html(&self) -> String {
        self.element_or_root().html()
    }

    pub fn attr<'js>(&self, ctx: Ctx<'js>, name: String) -> Result<Value<'js>> {
        match self.element().and_then(|e| e.value().attr(&name)) {
            Some(value) => value.into_js(&ctx),
            None => Ok(Value::new_null(ctx)),
        }
    }

    pub fn children(&self) -> Vec<HtmlNode> {
        self.element_or_root().children().filter_map(ElementRef::wrap).map(|e| self.wrap(e)).collect()
    }
}

pub(crate) fn install<'js>(ctx: &Ctx<'js>, state: &Rc<HostState>) -> Result<()> {
    let st = state.clone();
    let parse = Function::new(ctx.clone(), move |text: String| {
        let doc = Html::parse_document(&text);
        st.touch();
        let id = doc.tree.root().id();
        HtmlNode { doc: Rc::new(doc), id }
    })?;
    ctx.globals().set("__host_html_parse", parse)?;
    Ok(())
}
