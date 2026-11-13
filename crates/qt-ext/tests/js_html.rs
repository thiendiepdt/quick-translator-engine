mod common;

use qt_ext::testing;
use serde_json::json;

#[test]
fn boc_html_bang_css() {
    let src = r###"export async function run() {
      const d = html.parse(`<div id="toc"><ul><li><a href="/c/1">第1章 起</a></li><li class="vip"><a href="/c/2">第2章  承 </a></li></ul></div><p class="x">a<br>b</p>`);
      const links = d.select("#toc li a");
      let bad = null; try { d.select("##"); } catch (e) { bad = e.message; }
      const ul = d.selectFirst("ul");
      return {
        n: links.length, first: links[0].text(), second: links[1].text(), href: links[1].attr("href"),
        none: links[0].attr("nope"), p: d.selectFirst("p.x").html(), outer: d.selectFirst("p.x").outerHtml(),
        miss: d.selectFirst(".none"), kids: ul.children().length, inUl: ul.select("li.vip a").length,
        vip: ul.select("li").map(li => li.attr("class")), bad: bad !== null && bad.includes("css"),
        rootText: html.parse("<b>x</b>  <i>y</i>").text()
      };
    }"###;
    let pkg = common::package(&[("main.js", src)]);
    let v = testing::call(&pkg, "run", vec![], Default::default()).unwrap();
    assert_eq!(
        v,
        json!({ "n": 2, "first": "第1章 起", "second": "第2章 承", "href": "/c/2", "none": null,
                "p": "a<br>b", "outer": "<p class=\"x\">a<br>b</p>", "miss": null, "kids": 2, "inUl": 1,
                "vip": [null, "vip"], "bad": true, "rootText": "x y" })
    );
}
