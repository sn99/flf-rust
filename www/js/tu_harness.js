/**
 * TU identity harness — records match.game_state() each time unit (F.LF shape).
 * Enable: ?tu_dump=1 or window.__flf_tu_enable()
 * Download: window.__flf_tu_download()
 * Compare: /rust/tu_compare.html
 */
(function (w) {
  var enabled = false;
  var dump = [];
  var maxFrames = 900; // 30s @ 30 TU/s

  function qsEnable() {
    try {
      return new URLSearchParams(location.search).get('tu_dump') === '1';
    } catch (e) {
      return false;
    }
  }

  w.__flf_tu_enable = function (n) {
    enabled = true;
    dump = [];
    if (n) maxFrames = n;
    console.log('[tu_harness] enabled max=' + maxFrames);
  };
  w.__flf_tu_disable = function () {
    enabled = false;
  };
  w.__flf_tu_clear = function () {
    dump = [];
  };
  w.__flf_tu_get = function () {
    return dump.slice();
  };
  w.__flf_tu_record = function (state) {
    if (!enabled && !qsEnable()) return;
    if (!enabled) enabled = true;
    if (dump.length >= maxFrames) return;
    dump.push(state);
  };
  w.__flf_tu_download = function (filename) {
    var blob = new Blob([JSON.stringify(dump, null, 0)], { type: 'application/json' });
    var a = document.createElement('a');
    a.href = URL.createObjectURL(blob);
    a.download = filename || ('tu_dump_' + Date.now() + '.json');
    a.click();
    URL.revokeObjectURL(a.href);
    return dump.length;
  };
  /** Compare two dumps (arrays of game_state). Returns {ok, firstDiff, mismatches} */
  w.__flf_tu_compare = function (a, b) {
    var mismatches = [];
    var n = Math.min(a.length, b.length);
    for (var i = 0; i < n; i++) {
      var sa = a[i], sb = b[i];
      if (JSON.stringify(sa) !== JSON.stringify(sb)) {
        mismatches.push({ i: i, a: sa, b: sb });
        if (mismatches.length >= 20) break;
      }
    }
    if (a.length !== b.length) {
      mismatches.push({ i: -1, reason: 'length', a: a.length, b: b.length });
    }
    return { ok: mismatches.length === 0 && a.length === b.length, firstDiff: mismatches[0] || null, mismatches: mismatches, compared: n };
  };

  if (qsEnable()) w.__flf_tu_enable();
})(window);
