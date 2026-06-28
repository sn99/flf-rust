/**
 * Full AIin + AIcon for LF2_19 AI scripts (F.LF LF/AI.js parity).
 * Scripts are constructors with this.TU(); we keep instances and call TU each tick.
 * Loads AI/{name}.js or AI/{name}.json ({"_type":"function","source":"..."}).
 */
(function (global) {
  var sourceCache = {};
  var instances = {}; // key: uid|script -> { ai, controller, selfSnapUid }

  function makeController(uid) {
    var key = 'c' + uid;
    if (!instances._ctrls) instances._ctrls = {};
    if (!instances._ctrls[key]) {
      instances._ctrls[key] = {
        type: 'AIcontroller',
        state: {},
        buf: [],
        child: [],
        sync: true,
        key: function (k, down) {
          k = String(k);
          down = down ? 1 : 0;
          if (this.sync) {
            this.buf.push([k, down]);
          } else {
            this.state[k] = down;
          }
        },
        keypress: function (k, x, y) {
          k = String(k);
          // F.LF AIcon.keypress(key, x, y):
          // undefined/1,0 = tap; 1,1 = hold; 0,0 = release
          if ((x === undefined && y === undefined) || (x === 1 && y === 0)) {
            if (this.state[k]) this.key(k, 0);
            this.key(k, 1);
            this.key(k, 0);
          } else if (x === 1 && y === 1) {
            if (!this.state[k]) this.key(k, 1);
          } else if (x === 0 && y === 0) {
            if (this.state[k]) this.key(k, 0);
          }
        },
        keyseq: function (seq) {
          for (var i = 0; i < seq.length; i++) this.keypress(seq[i]);
        },
        clear_states: function () {
          for (var I in this.state) this.state[I] = 0;
        },
        flush: function () {
          this.buf.length = 0;
        },
        fetch: function () {
          for (var i = 0; i < this.buf.length; i++) {
            var I = this.buf[i][0],
              down = this.buf[i][1];
            this.state[I] = down;
          }
          this.buf.length = 0;
        },
        /** Drain pressed keys for Rust (held + taps this TU) */
        drain: function () {
          var out = [];
          var seen = {};
          for (var i = 0; i < this.buf.length; i++) {
            var k = this.buf[i][0],
              d = this.buf[i][1];
            if (d && !seen[k]) {
              seen[k] = 1;
              out.push(k);
            }
          }
          for (var sk in this.state) {
            if (this.state[sk] && !seen[sk]) {
              seen[sk] = 1;
              out.push(sk);
            }
          }
          this.fetch();
          return out;
        },
      };
    }
    return instances._ctrls[key];
  }

  function makeAI(snap) {
    var frameCache = { N: -1, O: {} };
    return {
      type: function () {
        return 0;
      },
      facing: function () {
        return snap.facing < 0 || snap.facing === 'left';
      },
      weapon_type: function () {
        if (!snap.hold_type) return 0;
        if (snap.hold_type === 'heavyweapon') return 2;
        if (snap.hold_type === 'drink') return 6;
        if (snap.hold_type === 'character') return -1;
        if (snap.hold_type === 'lightweapon') {
          // stand_throw property → 101
          var props = snap.properties || {};
          var oid = snap.hold_oid;
          var o = props[String(oid)] || props[oid];
          if (o && o.stand_throw) return 101;
          return 1;
        }
        return 1;
      },
      weapon_held: function () {
        return snap.hold_uid != null ? snap.hold_uid : -1;
      },
      weapon_holder: function () {
        return -1;
      },
      clone: function () {
        return -1;
      },
      blink: function () {
        return snap.blink ? Math.round((snap.effect_timeout || 0) / 2) : 0;
      },
      shake: function () {
        if (snap.oscillate) {
          var t = snap.effect_timeout || 0;
          return t * (snap.effect_dvx || snap.effect_dvy ? 1 : -1);
        }
        return 0;
      },
      ctimer: function () {
        return (snap.catch_counter || 0) * 6;
      },
      seqcheck: function (qe) {
        var seq = snap.combo_seq || [];
        if (!qe || !qe.length || !seq.length) return 0;
        var k1 = seq[seq.length - 1];
        if (k1 === qe[0]) return 1;
        if (seq.length < 2 || qe.length < 2) return 0;
        var k2 = seq[seq.length - 2];
        if (k2 === qe[0] && k1 === qe[1]) return 2;
        if (seq.length < 3 || qe.length < 3) return 0;
        var k3 = seq[seq.length - 3];
        if (k3 === qe[0] && k2 === qe[1] && k1 === qe[2]) return 3;
        return 0;
      },
      rand: function (i) {
        return Math.floor(Math.random() * i);
      },
      frame: function (N) {
        if (frameCache.N === N) return frameCache.O;
        frameCache.N = N;
        var O = (frameCache.O = {});
        var frames = snap.frames || {};
        var f = frames[String(N)] || frames[N];
        if (f) {
          for (var k in f) {
            if (!Object.prototype.hasOwnProperty.call(f, k)) continue;
            var v = f[k];
            if (k === 'itr' || k === 'bdy') {
              var arr = Array.isArray(v) ? v : v ? [v] : [];
              O[k + '_count'] = arr.length;
              O[k + 's'] = arr;
            } else if (k === 'wpoint' && v && typeof v === 'object') {
              O[k] = v;
            } else if (typeof v !== 'object' || v === null) {
              O[k] = v;
            } else if (k === 'opoint' || k === 'cpoint' || k === 'bpoint') {
              O[k] = v;
            }
          }
          if (O.itr_count === undefined) O.itr_count = 0;
          if (O.bdy_count === undefined) O.bdy_count = 0;
        } else {
          O.itr_count = 0;
          O.bdy_count = 0;
        }
        return O;
      },
      frame1: function () {
        return 0;
      },
    };
  }

  function updateSnapOnSelf(self, snap) {
    self.ps.x = snap.x;
    self.ps.y = snap.y;
    self.ps.z = snap.z;
    self.ps.vx = snap.vx || 0;
    self.ps.vy = snap.vy || 0;
    self.ps.vz = snap.vz || 0;
    self.ps.dir = snap.facing > 0 || snap.facing === 'right' ? 'right' : 'left';
    self.health.hp = snap.hp;
    self.health.mp = snap.mp;
    self.health.fall = snap.fall || 0;
    self.hp = snap.hp;
    self.mp = snap.mp;
    self.team = snap.team;
    self.id = snap.id;
    self.uid = snap.uid;
    self._state = snap.state;
    self.frame.N = snap.frame;
    self.frame.D =
      (snap.frames && (snap.frames[snap.frame] || snap.frames[String(snap.frame)])) || {};
    self.data.frame = snap.frames || self.data.frame;
    self.data.bmp = snap.bmp || self.data.bmp;
    self.hold.obj = snap.hold_type
      ? { type: snap.hold_type, id: snap.hold_oid || 0, uid: snap.hold_uid || -1 }
      : null;
    self.effect.blink = !!snap.blink;
    self.effect.timeout = snap.effect_timeout || 0;
    self.effect.oscillate = snap.oscillate || 0;
    self.effect.dvx = snap.effect_dvx || 0;
    self.effect.dvy = snap.effect_dvy || 0;
    self.effect.super = !!snap.super_armor;
    self.statemem.counter = snap.catch_counter || 0;
    self.combodec.seq = snap.combo_seq || [];
    self._snap = snap;
  }

  function makeSelf(snap) {
    var self = {
      ps: {
        x: snap.x,
        y: snap.y,
        z: snap.z,
        vx: snap.vx || 0,
        vy: snap.vy || 0,
        vz: snap.vz || 0,
        dir: snap.facing > 0 ? 'right' : 'left',
      },
      health: { hp: snap.hp, mp: snap.mp, fall: snap.fall || 0 },
      hp: snap.hp,
      mp: snap.mp,
      team: snap.team,
      id: snap.id,
      uid: snap.uid,
      type: 'character',
      _state: snap.state,
      state: function () {
        return this._state;
      },
      dirh: function () {
        return this.ps.dir === 'right' ? 1 : -1;
      },
      dirv: function () {
        return 0;
      },
      frame: {
        N: snap.frame,
        D: (snap.frames && (snap.frames[snap.frame] || snap.frames[String(snap.frame)])) || {},
      },
      data: {
        frame: snap.frames || {},
        bmp: snap.bmp || {
          running_speed: 9,
          walking_speed: 4,
          running_speedz: 2.5,
          walking_speedz: 2,
        },
      },
      hold: {
        obj: snap.hold_type
          ? { type: snap.hold_type, id: snap.hold_oid || 0, uid: snap.hold_uid || -1 }
          : null,
      },
      effect: {
        blink: !!snap.blink,
        timeout: snap.effect_timeout || 0,
        oscillate: snap.oscillate || 0,
        dvx: 0,
        dvy: 0,
        super: !!snap.super_armor,
      },
      catching: null,
      statemem: { counter: snap.catch_counter || 0 },
      combodec: { seq: snap.combo_seq || [] },
      proper: function (id, prop) {
        if (prop === undefined) {
          prop = id;
          id = snap.id;
        }
        var props = (this._snap && this._snap.properties) || snap.properties || {};
        var o = props[String(id)] || props[id];
        return o ? o[prop] : undefined;
      },
      AI: null,
      match: null,
      _snap: snap,
    };
    self.AI = makeAI(snap);
    // bind AI methods to use live snap via self
    var ai = self.AI;
    ai.facing = function () {
      return self.ps.dir === 'left';
    };
    ai.blink = function () {
      return self.effect.blink ? Math.round(self.effect.timeout / 2) : 0;
    };
    ai.shake = function () {
      if (self.effect.oscillate)
        return self.effect.timeout * (self.effect.dvx || self.effect.dvy ? 1 : -1);
      return 0;
    };
    ai.ctimer = function () {
      return (self.statemem.counter || 0) * 6;
    };
    ai.weapon_type = function () {
      if (!self.hold.obj) return 0;
      switch (self.hold.obj.type) {
        case 'lightweapon':
          if (self.proper(self.hold.obj.id, 'stand_throw')) return 101;
          return 1;
        case 'heavyweapon':
          return 2;
        case 'character':
          return -1;
        case 'drink':
          return 6;
      }
      return 0;
    };
    ai.weapon_held = function () {
      return self.hold.obj ? self.hold.obj.uid : -1;
    };
    ai.frame = function (N) {
      return makeAI(self._snap || snap).frame(N);
    };
    return self;
  }

  function makeMatch(snap, others) {
    var live = [makeSelf(snap)];
    (others || []).forEach(function (o) {
      live.push(makeSelf(o));
    });
    var m = {
      background: { width: snap.bg_w || 794, zboundary: snap.bg_z || [300, 450] },
      scene: {
        live: live,
        query: function () {
          return live;
        },
      },
      random: Math.random,
      get_living_object: function () {
        return live;
      },
      F6_mode: !!snap.f6_mode,
    };
    live.forEach(function (s) {
      s.match = m;
    });
    return m;
  }

  function extractSource(text, url) {
    if (!text) return '';
    text = text.trim();
    if (text.charAt(0) === '{') {
      try {
        var j = JSON.parse(text);
        if (j && j.source) return j.source;
        if (j && j._type === 'function' && j.source) return j.source;
      } catch (e) {}
    }
    return text;
  }

  function loadScript(root, name, cb) {
    if (sourceCache[name]) {
      cb(sourceCache[name]);
      return;
    }
    root = (root || '').replace(/\/?$/, '/');
    var urls = [root + 'AI/' + name + '.js', root + 'AI/' + name + '.json'];
    var i = 0;
    function next() {
      if (i >= urls.length) {
        cb('');
        return;
      }
      var u = urls[i++];
      fetch(u, { cache: 'no-cache' })
        .then(function (r) {
          return r.ok ? r.text() : Promise.reject();
        })
        .then(function (t) {
          var src = extractSource(t, u);
          if (src) {
            sourceCache[name] = src;
            cb(src);
          } else next();
        })
        .catch(next);
    }
    next();
  }

  function buildConstructor(source) {
    // Return factory (self, match, controller) -> instance with .TU
    try {
      var factory = new Function(
        'self',
        'match',
        'controller',
        'var define=function(a,b){var f=typeof a==="function"?a:b; try{return f();}catch(e){return null;}};' +
          'var print=function(){};' +
          'var abs=Math.abs, min=Math.min, max=Math.max, floor=Math.floor;' +
          source +
          ';\n' +
          'if (typeof AIscript === "function") return AIscript;\n' +
          'return null;'
      );
      // We need the AIscript function itself, not an instance
      var getter = new Function(
        'var define=function(a,b){var f=typeof a==="function"?a:b; try{return f();}catch(e){return null;}};' +
          'var print=function(){}; var abs=Math.abs,min=Math.min,max=Math.max,floor=Math.floor;' +
          source +
          ';\n return typeof AIscript==="function" ? AIscript : null;'
      );
      return getter();
    } catch (e) {
      console.warn('[ai_bridge] build fail', e);
      return null;
    }
  }

  var ctorCache = {};

  function getCtor(name, source) {
    if (ctorCache[name]) return ctorCache[name];
    var c = buildConstructor(source);
    if (c) ctorCache[name] = c;
    return c;
  }

  function runInstance(name, snap, others) {
    var source = sourceCache[name] || sourceCache.dumbass;
    if (!source) return [];
    var Ctor = getCtor(name, source) || getCtor('dumbass', sourceCache.dumbass);
    if (!Ctor) return [];
    var ikey = snap.uid + '|' + name;
    var ctrl = makeController(snap.uid);
    // reset taps but keep holds semantics via clear then TU
    ctrl.buf.length = 0;
    // release all at start of TU like many scripts do themselves
    var entry = instances[ikey];
    if (!entry) {
      var self0 = makeSelf(snap);
      var match0 = makeMatch(snap, others);
      self0.match = match0;
      // update others list on match for get_living_object
      try {
        entry = { ai: new Ctor(self0, match0, ctrl), self: self0, match: match0 };
      } catch (e1) {
        try {
          // some scripts not using `new`
          entry = { ai: Ctor(self0, match0, ctrl), self: self0, match: match0 };
        } catch (e2) {
          console.warn('[ai_bridge] construct', name, e2);
          return [];
        }
      }
      instances[ikey] = entry;
    } else {
      updateSnapOnSelf(entry.self, snap);
      // refresh living list
      var live = [entry.self];
      (others || []).forEach(function (o) {
        live.push(makeSelf(o));
      });
      entry.match.scene.live = live;
      entry.match.background.width = snap.bg_w || entry.match.background.width;
      entry.match.background.zboundary = snap.bg_z || entry.match.background.zboundary;
      entry.match.F6_mode = !!snap.f6_mode;
      live.forEach(function (s) {
        s.match = entry.match;
      });
    }
    try {
      if (entry.ai && typeof entry.ai.TU === 'function') entry.ai.TU();
    } catch (e) {
      console.warn('[ai_bridge] TU', name, e);
    }
    return ctrl.drain();
  }

  global.__flf_ai_preload = function (root, names) {
    root = (root || '').replace(/\/?$/, '/');
    names = names || ['dumbass', 'Challangar', 'Crusher', 'Ninja'];
    names.forEach(function (name) {
      loadScript(root, name, function (src) {
        if (src) console.log('[ai_bridge] loaded', name, src.length, 'chars');
      });
    });
  };

  global.__flf_ai_run = function (root, aiName, snapJson, othersJson) {
    var snap = typeof snapJson === 'string' ? JSON.parse(snapJson) : snapJson;
    var others = typeof othersJson === 'string' ? JSON.parse(othersJson) : othersJson || [];
    var name = aiName || 'dumbass';
    if (!sourceCache[name] && !sourceCache.dumbass) {
      // sync path: try XHR blocking is bad; kick async load and return []
      loadScript(root, name, function () {});
      loadScript(root, 'dumbass', function () {});
      return [];
    }
    if (!sourceCache[name]) name = 'dumbass';
    return runInstance(name, snap, others);
  };

  /** Force-sync load for tests */
  global.__flf_ai_load_sync = function (name, sourceText) {
    sourceCache[name] = extractSource(sourceText) || sourceText;
    delete ctorCache[name];
  };
})(typeof window !== 'undefined' ? window : globalThis);
