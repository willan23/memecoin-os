/**
 * MemeCoin OS Exchange widget.
 * Loads the official embed card in an iframe. No EXECUTE. No invented scores.
 *
 *   <div id="mcos"></div>
 *   <script src="https://memecoin-os.web.app/widget.js"></script>
 *   <script>
 *     MemeCoinOS.embed({ id: "pepe", target: "#mcos" });
 *   </script>
 */
(function (root) {
  var DEFAULT_ORIGIN = "https://memecoin-os.web.app";

  function originFromScript() {
    var scripts = document.getElementsByTagName("script");
    for (var i = scripts.length - 1; i >= 0; i--) {
      var src = scripts[i].src || "";
      if (src.indexOf("widget.js") !== -1) {
        try {
          return new URL(src).origin;
        } catch (e) {
          return DEFAULT_ORIGIN;
        }
      }
    }
    return DEFAULT_ORIGIN;
  }

  function resolveTarget(target) {
    if (!target) return null;
    if (typeof target === "string") return document.querySelector(target);
    return target;
  }

  function embed(opts) {
    opts = opts || {};
    var id = opts.id || opts.tokenId || opts.ecosystemId;
    var el = resolveTarget(opts.target);
    if (!id) throw new Error("MemeCoinOS.embed requires id");
    if (!el) throw new Error("MemeCoinOS.embed requires target");
    var origin = (opts.origin || originFromScript()).replace(/\/$/, "");
    var iframe = document.createElement("iframe");
    iframe.src = origin + "/embed/" + encodeURIComponent(id) + "/";
    iframe.width = String(opts.width || 380);
    iframe.height = String(opts.height || 340);
    iframe.setAttribute("loading", "lazy");
    iframe.setAttribute("title", "MemeCoin OS Intelligence");
    iframe.style.border = "0";
    iframe.style.borderRadius = "12px";
    iframe.style.background = "#070910";
    iframe.style.maxWidth = "100%";
    el.appendChild(iframe);
    return iframe;
  }

  root.MemeCoinOS = root.MemeCoinOS || {};
  root.MemeCoinOS.embed = embed;
})(typeof window !== "undefined" ? window : this);
