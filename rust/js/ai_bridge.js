/**
 * LF2_19 AI scripts for Rust WASM — synchronous run from preloaded cache.
 */
(function (global) {
  const cache = {}; // name -> source text
  const controllers = {};

  function makeController(uid) {
    if (!controllers[uid]) {
      controllers[uid] = {
        state: {},
        buf: [],
        keypress: function (key) {
          this.buf.push(String(key));
          this.state[key] = 1;
        },
        key: function (key, down) {
          this.state[key] = down ? 1 : 0;
          if (down) this.buf.push(String(key));
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

  function makeSelf(snap) {
    return {
      ps: {
        x: snap.x, y: snap.y, z: snap.z,
        vx: snap.vx, vy: snap.vy, vz: snap.vz,
        dir: snap.facing > 0 ? 'right' : 'left',
      },
      health: { hp: snap.hp, mp: snap.mp, fall: snap.fall || 0 },
      hp: snap.hp, mp: snap.mp, team: snap.team, id: snap.id, uid: snap.uid,
      type: 'character',
      state: function () { return snap.state; },
      frame: { N: snap.frame, D: {} },
      data: { frame: {}, bmp: { running_speed: 9, walking_speed: 4, running_speedz: 2.5, walking_speedz: 2 } },
      hold: {
        obj: snap.hold_type
          ? { type: snap.hold_type, id: snap.hold_oid || 0, uid: snap.hold_uid || -1 }
          : null,
      },
      effect: { blink: !!snap.blink, timeout: snap.effect_timeout || 0, oscillate: 0 },
      catching: null,
      statemem: { counter: snap.catch_counter || 0 },
      combodec: { seq: [] },
      proper: function () { return false; },
      AI: {
        type: function () { return 0; },
        facing: function () { return snap.facing < 0; },
        weapon_type: function () {
          if (!snap.hold_type) return 0;
          if (snap.hold_type === 'heavyweapon') return 2;
          if (snap.hold_type === 'drink') return 6;
          return 1;
        },
        weapon_held: function () { return snap.hold_uid || -1; },
        blink: function () { return snap.blink ? 1 : 0; },
        frame: function () { return { state: snap.state }; },
        rand: function (i) { return Math.floor(Math.random() * i); },
        ctimer: function () { return snap.catch_counter || 0; },
        seqcheck: function () { return 0; },
      },
    };
  }

  function makeMatch(snap, others) {
    var live = [makeSelf(snap)].concat((others || []).map(makeSelf));
    return {
      background: {
        width: snap.bg_w || 794,
        zboundary: snap.bg_z || [300, 450],
      },
      scene: { live: live },
      random: Math.random,
      get_living_object: function () { return live; },
    };
  }

  function runText(text, self, match, controller) {
    try {
      var fn = new Function(
        'self',
        'match',
        'controller',
        'var define=function(a,b){var f=typeof a==="function"?a:b; try{return f();}catch(e){return null;}};' +
          text +
          ';try{if(typeof AIscript==="function"){try{new AIscript(self,match,controller);}catch(e1){AIscript(self,match,controller);}}}catch(e2){}'
      );
      fn(self, match, controller);
    } catch (e) {
      /* script errors — fall back to Rust heuristics */
    }
  }

  /** Preload AI scripts (call once with asset root) */
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

  /** Synchronous AI TU — returns string[] keys */
  global.__flf_ai_run = function (root, aiName, snapJson, othersJson) {
    var snap = typeof snapJson === 'string' ? JSON.parse(snapJson) : snapJson;
    var others = typeof othersJson === 'string' ? JSON.parse(othersJson) : othersJson || [];
    var name = aiName || 'dumbass';
    var text = cache[name] || cache.dumbass;
    if (!text) return [];
    var ctrl = makeController(snap.uid);
    ctrl.clear_states();
    runText(text, makeSelf(snap), makeMatch(snap, others), ctrl);
    return ctrl.drain();
  };
})(typeof window !== 'undefined' ? window : globalThis);
