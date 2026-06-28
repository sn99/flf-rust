/**
 * F.LF core/network lockstep application layer (JS port).
 * Message shape: { f: { t, d } } frames, { m: mess } messenger, { t: { name, data } } transfer.
 * Transport plugs via conn.send / ondata; PeerJS or BroadcastChannel adapters below.
 */
(function (w) {
  var This = null;
  function reset() {
    This = {
      already: false,
      conn: null,
      time: 0,
      timer: null,
      timer_callback: null,
      target_interval: 0,
      lasttime: Date.now(),
      frame: { buffer: [] },
      transfer: { obj: {} },
      messenger: {},
      monitor: { on: function () {} }
    };
  }
  reset();

  function set_interval(cb, intervalMs) {
    if (This.timer_callback) {
      console.error('[flf network] only one timer; clearInterval first');
      return;
    }
    This.timer_callback = cb;
    This.target_interval = intervalMs;
    This.timer = setInterval(frameTick, This.target_interval * 0.5);
    return This.timer;
  }
  function clear_interval(id) {
    if (!This.timer || This.timer !== id) return;
    clearInterval(This.timer);
    This.timer = null;
    This.timer_callback = null;
  }
  function frameTick() {
    if (!This.timer_callback) return;
    if (!This.frame.buffer[0]) return;
    var newtime = Date.now();
    var diff = newtime - This.lasttime;
    if (diff > This.target_interval - 5) {
      if (This.frame.buffer[0].time !== This.time) {
        This.monitor.on('sync_error');
      }
      This.time++;
      This.timer_callback(This.frame.buffer[0].time, This.frame.buffer[0].data, channels.frame.send);
      This.lasttime = newtime;
      This.frame.buffer.shift();
    }
  }
  var channels = {
    frame: {
      send: function (data) {
        if (!This.conn || !This.conn.send) return;
        This.conn.send({ f: { t: This.time, d: data } });
      },
      receive: function (data) {
        This.frame.buffer.push({ time: data.t, data: data.d });
        frameTick();
      }
    },
    messenger: {
      send: sender('messenger'),
      receive: function (mess) {
        if (This.messenger.receiver && This.messenger.receiver.onmessage)
          This.messenger.receiver.onmessage(mess);
      }
    },
    transfer: {
      send: sender('transfer'),
      receive: function (data) {
        var name = data.name;
        var receive = This.transfer.obj[name];
        if (!receive) {
          console.error('[flf network] no receiver for', name);
          return;
        }
        receive(data.data);
        This.transfer.obj[name] = null;
      }
    }
  };
  function sender(name) {
    var key = name.charAt(0);
    return function (data) {
      if (!This.conn || !This.conn.send) return;
      var obj = {};
      obj[key] = data;
      This.conn.send(obj);
    };
  }
  function transfer(name, sendFn, receiveFn) {
    if (This.transfer.obj[name]) {
      console.error('[flf network] name in use', name);
      return;
    }
    This.transfer.obj[name] = receiveFn;
    channels.transfer.send({ name: name, data: sendFn() });
  }
  function onTransportData(data) {
    if (!data || typeof data !== 'object') {
      try { data = JSON.parse(data); } catch (e) { return; }
    }
    for (var ch in channels) {
      var c = ch.charAt(0);
      if (data[c]) channels[ch].receive(data[c]);
    }
  }

  /** Lightweight PeerJS / BroadcastChannel transport when F.Lobby library unavailable */
  function setupTransport(config, handler) {
    var room = (config.param && (config.param.room || config.param.id1)) || 'default';
    room = String(room).replace(/[^a-zA-Z0-9_-]/g, '') || 'default';
    var isActive = !config.param || config.param.role !== 'passive';
    var hostId = 'flf-' + room;
    var inbox = [];
    var connObj = {
      send: function (payload) {
        var s = typeof payload === 'string' ? payload : JSON.stringify(payload);
        if (w.__flf_conn && w.__flf_conn.open) {
          try { w.__flf_conn.send(payload); } catch (e) {}
        }
        if (w.__flf_bc) {
          try { w.__flf_bc.postMessage(payload); } catch (e) {}
        }
        // also feed legacy peer glue inbox path for Rust consumer
        if (typeof w.__flf_peer_send === 'function') {
          try { w.__flf_peer_send(s); } catch (e) {}
        }
      }
    };

    // BroadcastChannel always
    try {
      var chName = 'flf-lockstep-' + (config.param && config.param.id1 ? config.param.id1 : room);
      w.__flf_bc = new BroadcastChannel(chName);
      w.__flf_bc.onmessage = function (ev) {
        handler.on('data', ev.data);
      };
    } catch (e) {}

    function openConn(c) {
      w.__flf_conn = c;
      This.conn = connObj;
      c.on('data', function (d) {
        handler.on('data', typeof d === 'string' ? (function () { try { return JSON.parse(d); } catch (e) { return d; } })() : d);
      });
      c.on('open', function () {
        This.messenger.send = channels.messenger.send;
        channels.frame.send();
        handler.on('open', connObj);
      });
    }

    if (w.Peer) {
      try {
        var peer = new w.Peer(isActive ? hostId : undefined);
        w.__flf_peer = peer;
        peer.on('open', function (id) {
          handler.on('log', 'peer open ' + id);
          if (!isActive) {
            var c = peer.connect(hostId, { reliable: true });
            openConn(c);
          }
        });
        peer.on('connection', function (c) {
          openConn(c);
        });
        peer.on('error', function (e) {
          handler.on('error', String(e && e.type || e));
          // fall back: treat BC as open
          if (!This.conn) {
            This.conn = connObj;
            This.messenger.send = channels.messenger.send;
            channels.frame.send();
            handler.on('open', connObj);
            handler.on('log', 'PeerJS error; BroadcastChannel lockstep active');
          }
        });
      } catch (e) {
        handler.on('error', String(e));
      }
    }

    // If no Peer or passive waiting, open BC immediately for multi-tab
    setTimeout(function () {
      if (!This.conn) {
        This.conn = connObj;
        This.messenger.send = channels.messenger.send;
        channels.frame.send();
        handler.on('open', connObj);
        handler.on('log', 'BroadcastChannel lockstep open room=' + room);
      }
    }, 800);
  }

  function setup(config, monitor) {
    if (This.already) {
      console.error('[flf network] setup already');
      return;
    }
    This.already = true;
    This.monitor = monitor || { on: function () {} };
    var handler = {
      on: function (event, data) {
        switch (event) {
          case 'open':
            This.conn = data;
            This.messenger.send = channels.messenger.send;
            channels.frame.send();
            This.monitor.on('open');
            break;
          case 'close':
            This.monitor.on('close');
            break;
          case 'log':
            This.monitor.on('log', data);
            break;
          case 'error':
            This.monitor.on('error', data);
            break;
          case 'data':
            onTransportData(data);
            break;
        }
      }
    };

    // Prefer remote F.Lobby transport library if server.library provided
    var lib = config.server && config.server.library;
    var addr = config.server && (config.server.address || config.server);
    if (lib && addr && typeof define === 'function') {
      // AMD not loaded in rust shell — skip
    }
    if (lib && addr) {
      var base = String(addr);
      if (base.charAt(base.length - 1) !== '/') base += '/';
      var scriptUrl = base + lib.replace(/^\//, '');
      // Dynamic script load attempt (may fail CORS / mixed content)
      var s = document.createElement('script');
      s.src = scriptUrl;
      s.onerror = function () {
        This.monitor.on('log', 'F.Lobby transport library failed; using Peer/BC');
        setupTransport(config, handler);
      };
      s.onload = function () {
        if (w.flobby_transport && w.flobby_transport.setup) {
          w.flobby_transport.setup(config, handler);
        } else {
          setupTransport(config, handler);
        }
      };
      document.head.appendChild(s);
      // timeout fallback
      setTimeout(function () {
        if (!This.conn) setupTransport(config, handler);
      }, 2000);
    } else {
      setupTransport(config, handler);
    }

    w.__flf_network_config = config;
  }

  function teardown() {
    if (This.timer) clearInterval(This.timer);
    reset();
  }

  // LF/network application layer on top: control buffers + verify
  var local = [];
  var remote = [];
  var verify = null;
  var packet = null;
  var appCallback = null;
  var appHandler = null;

  function appSetInterval(cb, intMs) {
    verify = {};
    packet = { control: [] };
    appCallback = cb;
    return set_interval(appFrame, intMs);
  }
  function appClearInterval(t) {
    clear_interval(t);
    verify = packet = appCallback = null;
  }
  function appFrame(time, data, send) {
    if (data && data.control) {
      for (var i = 0; i < remote.length; i++) {
        if (remote[i]) remote[i].supply(data.control[i]);
      }
    }
    for (var j = 0; j < local.length; j++) {
      packet.control[j] = local[j].pre_fetch();
    }
    packet.verify = verify.last;
    send(packet);
    appCompare(verify.last_last, data && data.verify);
    verify.last_last = verify.last;
    verify.last = appCallback ? appCallback() : null;
    for (var k = 0; k < local.length; k++) local[k].swap_buffer();
    if (packet) packet.control.length = 0;
  }
  function appCompare(A, B) {
    if (A === undefined || B === undefined) return;
    for (var I in A) {
      if (!same(A[I], B[I])) {
        if (!verify.error) {
          if (appHandler) appHandler.on('sync_error');
          console.log('[flf network] sync mismatch', A, B);
          verify.error = true;
        }
      }
    }
    function same(a, b) {
      if (typeof a !== typeof b) return false;
      if (typeof a === 'object' && a && b) {
        for (var i in a) if (a[i] !== b[i]) return false;
        return true;
      }
      return a === b;
    }
  }

  function NCon(role, control) {
    this.state = {};
    this.child = [];
    this.buf = [];
    this.pre_buf = [];
    this.sync = true;
    this.role = role;
    if (role === 'local' || role === 'dual') {
      local.push(this);
      this.wrap(control);
      if (control && control.child) control.child.push(this);
      if (control) control.sync = true;
      if (control && control.state) {
        for (var i in control.state) this.state[i] = 0;
      }
    }
    if (role === 'remote' || role === 'dual') {
      remote.push(this);
      if (role === 'remote' && control) {
        for (var j in control) this.state[j] = 0;
      }
    }
  }
  NCon.prototype.wrap = function (control) {
    this.control = control;
    if (!control) return;
    this.type = control.type;
    this.config = control.config;
  };
  NCon.prototype.clear_states = function () {};
  NCon.prototype.flush = function () {};
  NCon.prototype.pre_fetch = function () {
    if (this.role === 'local' || this.role === 'dual') {
      if (this.control && this.control.fetch) this.control.fetch();
      return this.pre_buf;
    }
  };
  NCon.prototype.swap_buffer = function () {
    if (this.role === 'local' || this.role === 'dual') {
      var hold = this.pre_buf;
      this.pre_buf = this.buf;
      this.buf = hold;
      this.pre_buf.length = 0;
    }
  };
  NCon.prototype.supply = function (buf) {
    if (this.role === 'remote' || this.role === 'dual') {
      if (buf && buf.length) this.buf = this.buf.concat(buf);
    }
  };
  NCon.prototype.fetch = function () {
    for (var i = 0; i < this.buf.length; i++) {
      var I = this.buf[i];
      var K = I[0], D = I[1];
      for (var j = 0; j < this.child.length; j++) this.child[j].key(K, D);
      this.state[K] = D;
    }
    this.buf.length = 0;
  };
  NCon.prototype.key = function (K, down) {
    this.pre_buf.push([K, down]);
  };

  function appSetup(config, handler) {
    appHandler = handler;
    local.length = 0;
    remote.length = 0;
    setup(config, handler);
  }

  w.__flf_network = {
    setup: appSetup,
    teardown: teardown,
    setInterval: appSetInterval,
    clearInterval: appClearInterval,
    transfer: transfer,
    messenger: This.messenger,
    controller: NCon,
    core: { setup: setup, setInterval: set_interval, clearInterval: clear_interval, transfer: transfer },
    onTransportData: onTransportData
  };
  // compatibility alias
  w.FLFNetwork = w.__flf_network;
})(window);
