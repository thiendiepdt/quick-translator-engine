// Extension mẫu: nguồn có API JSON, cần API key.
//
// API (giả định):
//   POST /auth/token { apiKey, device }   → { token, expiresIn }   (401 nếu key sai)
//   GET  /books/<mã>/chapters?page=N&lang=… → { pages, items: [{ id, title, vip, purchased }] }
//   GET  /chapters/<mã>?lang=…              → { title, content }   (401 token hết hạn, 403 chương VIP)
// Mọi request có header Authorization: Bearer <token> và X-Device: <mã thiết bị>.

const TOC_TTL = 600; // giây

function base(ctx) {
  return String(ctx.settings.apiBase || "https://api.truyen-mau.example").replace(/\/$/, "");
}

function bookId(ctx) {
  const found = ctx.rawUrl.match(/\/novel\/(\d+)/);
  if (!found) throw new ExtensionError("link tải raw phải có dạng …/novel/<số>");
  return found[1];
}

/** Mã thiết bị ngẫu nhiên, tạo một lần rồi giữ trong session data (mất khi đổi API key hoặc Đăng xuất). */
async function deviceId() {
  let id = await session.get("deviceId");
  if (!id) {
    id = encoding.hex.encode(crypto.randomBytes(8));
    await session.set("deviceId", id);
  }
  return id;
}

async function api(ctx, path) {
  const res = await http.get(base(ctx) + path, {
    headers: { Authorization: "Bearer " + ctx.session.token, "X-Device": await deviceId() },
  });
  // App xoá phiên, gọi login rồi chạy lại đúng lệnh này một lần.
  if (res.status === 401) throw new AuthExpired("token hết hạn");
  if (res.status === 403) throw new Locked("chương VIP chưa mua");
  if (res.status >= 500) throw new ExtensionError("máy chủ lỗi " + res.status, { retry: true });
  if (!res.ok) throw new ExtensionError("API trả HTTP " + res.status);
  return res.json();
}

// App gọi trước lượt tải khi chưa có phiên hoặc phiên hết hạn; kết quả đưa lại qua ctx.session.
export async function login(ctx) {
  const res = await http.post(base(ctx) + "/auth/token", { apiKey: ctx.auth.values.apiKey, device: await deviceId() });
  if (res.status === 401) throw new ExtensionError("API key sai");
  if (!res.ok) throw new ExtensionError("đăng nhập trả HTTP " + res.status, { retry: true });
  const body = res.json();
  return { token: body.token, expiresAt: Date.now() + body.expiresIn * 1000 };
}

export async function toc(ctx) {
  const key = "toc:" + bookId(ctx) + ":" + ctx.settings.lang;
  const cached = await cache.get(key);
  if (cached) {
    log.info("mục lục lấy từ cache (" + cached.length + " chương)");
    return cached;
  }
  const items = [];
  for (let page = 1; ; page++) {
    const body = await api(ctx, "/books/" + bookId(ctx) + "/chapters?page=" + page + "&lang=" + ctx.settings.lang);
    for (const c of body.items) items.push({ id: String(c.id), title: c.title, locked: !!c.vip && !c.purchased });
    progress(page, body.pages);
    if (page >= body.pages) break;
  }
  await cache.set(key, items, { ttl: TOC_TTL });
  return items;
}

export async function chapter(ctx, ch) {
  const body = await api(ctx, "/chapters/" + ch.id + "?lang=" + ctx.settings.lang);
  return { content: body.content, title: body.title };
}
