/**
 * Full AIin-style interface for LF2_19 AI scripts (F.LF AI.js parity subset).
 */
(function (global) {
  var cache = {};
  var controllers = {};

  function makeController(uid) {
    if (!controllers[uid]) {
      controllers[uid] = {
        state: {},
        buf: [],
        sync: true,
        keypress: function (key, x, y) {
          key = String(key);
          this.buf.push(key);
          this.state[key] = 1;
        },
        key: function (key, down) {
          key = String(key);
          this.state[key] = down ? 1 : 0;
          if (down) this.buf.push(key);
        },
        keyseq: function (seq) {
          for (var i = 0; i < seq.length; i++) this.keypress(seq[i]);
        },
        clear_states: function () {
          this.state = {};
          this.buf = [];
        },
        drain: function () {
          var b = this.buf.slice();
          this.buf = [];
          return b;
        },
      };
    }
    return controllers[uid];
  }

  function makeAI(snap) {
    var frameCache = { N: -1, O: {} };
    return {
      type: function () { return 0; },
      facing: function () { return snap.facing < 0; },
      weapon_type: function () {
        if (!snap.hold_type) return 0;
        if (snap.hold_type === 'heavyweapon') return 2;
        if (snap.hold_type === 'drink') return 6;
        if (snap.hold_type === 'character') return -1;
        return 1;
      },
      weapon_held: function () { return snap.hold_uid != null ? snap.hold_uid : -1; },
      weapon_holder: function () { return -1; },
      clone: function () { return -1; },
      blink: function () {
        return snap.blink ? Math.round((snap.effect_timeout || 0) / 2) : 0;
      },
      shake: function () { return 0; },
      ctimer: function () { return snap.catch_counter || 0; },
      seqcheck: function () { return 0; },
      rand: function (i) { return Math.floor(Math.random() * i); },
      frame: function (N) {
        if (frameCache.N === N) return frameCache.O;
        frameCache.N = N;
        var O = (frameCache.O = {});
        var frames = snap.frames || {};
        var f = frames[String(N)] || frames[N];
        if (f) {
          for (var k in f) {
            if (Object.prototype.hasOwnProperty.call(f, k)) O[k] = f[k];
          }
          if (f.itr) {
            O.itr_count = Array.isArray(f.itr) ? f.itr.length : 1;
            O.itrs = Array.isArray(f.itr) ? f.itr : [f.itr];
          } else O.itr_count = 0;
          if (f.bdy) {
            O.bdy_count = Array.isArray(f.bdy) ? f.bdy.length : 1;
            O.bdys = Array.isArray(f.bdy) ? f.bdy : [f.bdy];
          } else O.bdy_count = 0;
        } else {
          O.itr_count = 0;
          O.bdy_count = 0;
        }
        return O;
      },
      frame1: function () { return 0; },
    };
  }

  function makeSelf(snap) {
    var self = {
      ps: {
        x: snap.x, y: snap.y, z: snap.z,
        vx: snap.vx || 0, vy: snap.vy || 0, vz: snap.vz || 0,
        dir: snap.facing > 0 ? 'right' : 'left',
      },
      health: { hp: snap.hp, mp: snap.mp, fall: snap.fall || 0 },
      hp: snap.hp, mp: snap.mp, team: snap.team, id: snap.id, uid: snap.uid,
      type: 'character',
      state: function () { return snap.state; },
      frame: { N: snap.frame, D: (snap.frames && (snap.frames[snap.frame] || snap.frames[String(snap.frame)])) || {} },
      data: { frame: snap.frames || {}, bmp: snap.bmp || { running_speed: 9, walking_speed: 4, running_speedz: 2.5, walking_speedz: 2 } },
      hold: {
        obj: snap.hold_type
          ? { type: snap.hold_type, id: snap.hold_oid || 0, uid: snap.hold_uid || -1 }
          : null,
      },
      effect: { blink: !!snap.blink, timeout: snap.effect_timeout || 0, oscillate: 0, super: !!snap.super_armor },
      catching: null,
      statemem: { counter: snap.catch_counter || 0 },
      combodec: { seq: snap.combo_seq || [] },
      proper: function (id, prop) {
        if (prop === undefined) { prop = id; id = snap.id; }
        var props = snap.properties || {};
        var o = props[String(id)] || props[id];
        return o ? o[prop] : undefined;
      },
      AI: makeAI(snap),
      match: null,
    };
    return self;
  }

  function makeMatch(snap, others) {
    var live = [makeSelf(snap)].concat((others || []).map(makeSelf));
    live.forEach(function (s) { s.AI = makeAI(Object.assign({}, snap, { uid: s.uid, x: s.ps.x, z: s.ps.z, team: s.team })); });
    var m = {
      background: { width: snap.bg_w || 794, zboundary: snap.bg_z || [300, 450] },
      scene: { live: live, query: function () { return live; } },
      random: Math.random,
      get_living_object: function () { return live; },
      F6_mode: !!snap.f6_mode,
    };
    live.forEach(function (s) { s.match = m; });
    return m;
  }

  function runText(text, self, match, controller) {
    try {
      var fn = new Function(
        'self', 'match', 'controller',
        'var define=function(a,b){var f=typeof a==="function"?a:b; try{return f();}catch(e){return null;}};' +
          text +
          ';try{if(typeof AIscript==="function"){try{new AIscript(self,match,controller);}catch(e1){AIscript(self,match,controller);}}}catch(e2){}'
      );
      fn(self, match, controller);
    } catch (e) { /* fall back to Rust */ }
  }

  global.__flf_ai_preload = function (root, names) {
    root = (root || '').replace(/\/?$/, '/');
    names = names || ['dumbass', 'Challangar', 'Crusher', 'Ninja'];
    names.forEach(function (name) {
      fetch(root + 'AI/' + name + '.js')
        .then(function (r) { return r.ok ? r.text() : ''; })
        .then(function (t) { if (t) cache[name] = t; })
        .catch(function () {});
    });
  };

  global.__flf_ai_run = function (root, aiName, snapJson, othersJson) {
    var snap = typeof snapJson === 'string' ? JSON.parse(snapJson) : snapJson;
    var others = typeof othersJson === 'string' ? JSON.parse(othersJson) : othersJson || [];
    var name = aiName || 'dumbass';
    var text = cache[name] || cache.dumbass;
    if (!text) return [];
    var ctrl = makeController(snap.uid);
    ctrl.clear_states();
    var self = makeSelf(snap);
    var match = makeMatch(snap, others);
    self.match = match;
    runText(text, self, match, ctrl);
    return ctrl.drain();
  };
})(typeof window !== 'undefined' ? window : globalThis);
