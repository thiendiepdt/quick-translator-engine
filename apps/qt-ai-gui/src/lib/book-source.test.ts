import { describe, expect, it } from "vitest";

import { baiduCovers, detectBookSource, fanqieCovers, parseSourceBook } from "@/lib/book-source";

describe("detectBookSource", () => {
  it("nhận link nguồn thư viện đang dùng, ra đúng URL tải", () => {
    expect(detectBookSource("https://book.qq.com/book-detail/43422902")).toEqual({
      source: "qq",
      bookId: "43422902",
      fetchUrl: "https://detailadr.reader.qq.com/book/queryDetailPage?bid=43422902&sex=1&qmk=1,3,7,9,10",
    });
    expect(detectBookSource("https://fanqienovel.com/page/7077360447967464485")?.fetchUrl).toBe(
      "https://fanqienovel.com/page/7077360447967464485",
    );
    expect(detectBookSource("https://www.qidian.com/book/1039059594/")?.fetchUrl).toBe("https://m.qidian.com/book/1039059594/");
    expect(detectBookSource("https://www.qimao.com/shuku/10021482/")?.bookId).toBe("10021482");
    expect(detectBookSource("https://book.sfacg.com/Novel/666530")?.fetchUrl).toBe("https://m.sfacg.com/b/666530/");
    expect(detectBookSource("https://www.jjwxc.net/onebook.php?novelid=10422146")?.fetchUrl).toBe(
      "https://app.jjwxc.net/androidapi/novelbasicinfo?novelId=10422146",
    );
  });

  it("QQ bỏ offset 1_000_000_000; mirror sangtacviet theo đường dẫn", () => {
    expect(detectBookSource("https://book.qq.com/book-detail/1059366432")?.bookId).toBe("59366432");
    expect(detectBookSource("https://sangtacviet.vip/truyen/fanqie/1/7077360447967464485/")?.source).toBe("fanqie");
  });

  it("link không hỗ trợ hoặc rỗng → null", () => {
    expect(detectBookSource("https://www.piaotia.com/html/7/7028/")).toBeNull();
    expect(detectBookSource("https://www.novel543.com/0924601016/")).toBeNull();
    expect(detectBookSource("")).toBeNull();
    expect(detectBookSource("không phải link")).toBeNull();
  });
});

describe("parseSourceBook", () => {
  it("QQ: tác giả và ảnh trong dữ liệu trước, CDN dựng từ bid sau", () => {
    const body = JSON.stringify({
      introinfo: { book: { title: "方寸道主", author: "愤怒的乌贼", coverUrl: "https://ccstatic.example.com/a.jpg" } },
    });
    const ref = detectBookSource("https://book.qq.com/book-detail/58257836")!;
    expect(parseSourceBook(ref, body)).toEqual({
      source: "qq",
      title: "方寸道主",
      author: "愤怒的乌贼",
      coverUrls: ["https://ccstatic.example.com/a.jpg", "https://bookcover.reader.qq.com/cover/836/58257836/t9_58257836.webp"],
    });
    expect(() => parseSourceBook(ref, JSON.stringify({ msg: "书籍不存在", introinfo: {} }))).toThrow("书籍不存在");
  });

  it("Fanqie: đọc __INITIAL_STATE__ (literal JS), ảnh đổi sang host không chữ ký", () => {
    const html = `<script>window.__INITIAL_STATE__ = {"page":{"bookName":"书","authorName":"作者A","thumbUri":"https://p6-novel-sign.byteimg.com/novel-pic/abc~tplv-resize:225:300.image?x-expires=1","x":undefined,},};</script>`;
    const book = parseSourceBook(detectBookSource("https://fanqienovel.com/page/1")!, html);
    expect(book.author).toBe("作者A");
    expect(book.coverUrls).toEqual([
      "https://p6-novel.byteimg.com/novel-pic/abc~tplv-shrink:640:0.image",
      "https://p6-novel.byteimg.com/origin/novel-pic/abc",
    ]);
    expect(() => parseSourceBook(detectBookSource("https://fanqienovel.com/page/1")!, "<html></html>")).toThrow(/Fanqie/);
  });

  it("Qidian: page context JSON, ảnh dựng từ id", () => {
    const html = `<script id="vite-plugin-ssr_pageContext" type="application/json">{"pageContext":{"pageProps":{"pageData":{"bookInfo":{"bookName":"书","authorName":"作者B"}}}}}</script>`;
    const book = parseSourceBook(detectBookSource("https://www.qidian.com/book/1039059594/")!, html);
    expect(book.author).toBe("作者B");
    expect(book.coverUrls).toEqual(["https://bookcover.yuewen.com/qdbimg/349573/1039059594/600"]);
  });

  it("Qimao và SFACG: đọc markup; ảnh // thành https", () => {
    const qimao = `<div class="book-information"><div class="wrap-pic"><img src="//cdn.wtzw.com/x/abc_360x480.png"></div><div class="title"><span class="txt">书</span></div><div class="sub-title"><span class="txt">作者：<em>作者C</em></span></div></div>`;
    const book = parseSourceBook(detectBookSource("https://www.qimao.com/shuku/10021482/")!, qimao);
    expect(book.author).toBe("作者C");
    expect(book.coverUrls).toEqual(["https://cdn.wtzw.com/x/abc_360x480.png", "https://cdn.wtzw.com/x/abc.png"]);

    const sfacg = `<div class="book_newtitle">书</div><div class="book_info"><img src="http://rs.sfacg.com/c.jpg"></div><div class="book_info3">作者D / 100万字</div>`;
    const sf = parseSourceBook(detectBookSource("https://book.sfacg.com/Novel/666530")!, sfacg);
    expect(sf).toMatchObject({ author: "作者D", coverUrls: ["https://rs.sfacg.com/c.jpg"] });
  });

  it("JJWXC: JSON app, ảnh Baidu thử bản thu nhỏ trước", () => {
    const body = JSON.stringify({ novelName: "书", authorName: "作者E", novelCover: "http://pic.rmb.bdstatic.com/a.jpeg" });
    const book = parseSourceBook(detectBookSource("https://www.jjwxc.net/onebook.php?novelid=10422146")!, body);
    expect(book.author).toBe("作者E");
    expect(book.coverUrls).toEqual(["https://pic.rmb.bdstatic.com/a.jpeg@w_600,q_80", "https://pic.rmb.bdstatic.com/a.jpeg"]);
  });
});

describe("cover helpers", () => {
  it("fanqieCovers và baiduCovers giữ link lạ nguyên vẹn", () => {
    expect(fanqieCovers("")).toEqual([]);
    expect(baiduCovers("https://i4-static.jjwxc.net/a.jpg")).toEqual(["https://i4-static.jjwxc.net/a.jpg"]);
    expect(baiduCovers("https://pic.rmb.bdstatic.com/a.jpeg@w_300")).toEqual(["https://pic.rmb.bdstatic.com/a.jpeg@w_300"]);
  });
});
