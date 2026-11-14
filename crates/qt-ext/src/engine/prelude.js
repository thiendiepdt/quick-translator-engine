// Chạy trước main.js của extension. Gói hàm `__host_*` do app gắn thành API công khai rồi xoá `__host_*`
// khỏi globalThis để extension chỉ dùng qua lớp này. API gắn cố định (không ghi đè, không xoá được).
(() => {
  const g = globalThis;
  const host = {};
  for (const key of Object.getOwnPropertyNames(g)) {
    if (key.startsWith("__host_")) {
      host[key.slice(7)] = g[key];
      delete g[key];
    }
  }

  const freeze = Object.freeze;
  const isFrozen = Object.isFrozen;
  const names = Object.getOwnPropertyNames;
  const define = (name, value) =>
    Object.defineProperty(g, name, { value, writable: false, configurable: false, enumerable: false });

  function deepFreeze(value) {
    if (value !== null && typeof value === "object" && !isFrozen(value)) {
      freeze(value);
      for (const key of names(value)) deepFreeze(value[key]);
    }
    return value;
  }

  class AuthExpired extends Error {
    constructor(message) { super(message); this.name = "AuthExpired"; }
  }
  class Locked extends Error {
    constructor(message) { super(message); this.name = "Locked"; }
  }
  class NeedsInteraction extends Error {
    constructor(message) { super(message); this.name = "NeedsInteraction"; }
  }
  class ExtensionError extends Error {
    constructor(message, options) {
      super(message);
      this.name = "ExtensionError";
      this.retry = !!(options && options.retry);
    }
  }
  define("AuthExpired", AuthExpired);
  define("Locked", Locked);
  define("NeedsInteraction", NeedsInteraction);
  define("ExtensionError", ExtensionError);

  const show = (value) => {
    if (typeof value === "string") return value;
    if (value instanceof Error) return value.name + ": " + value.message;
    try { return JSON.stringify(value); } catch (e) { return String(value); }
  };
  const format = (args) => args.map(show).join(" ");
  const log = freeze({
    info: (...args) => host.log("info", format(args)),
    warn: (...args) => host.log("warn", format(args)),
    error: (...args) => host.log("error", format(args)),
  });
  define("log", log);
  define("console", freeze({ log: log.info, info: log.info, warn: log.warn, error: log.error }));
  // Tham số tuỳ chọn phía Rust không nhận `undefined` truyền tường minh.
  define("progress", (done, total, message) =>
    message == null
      ? host.progress(Number(done) || 0, Number(total) || 0)
      : host.progress(Number(done) || 0, Number(total) || 0, String(message)));
  define("sleep", (ms) => host.sleep(Math.max(0, Math.floor(Number(ms) || 0))));

  // Lớp JS của từng nhóm API (http, html, crypto, encoding) chèn vào đây lúc dựng engine.
  /*BINDINGS*/

  define("__qt_internal", freeze({ deepFreeze }));
})();
