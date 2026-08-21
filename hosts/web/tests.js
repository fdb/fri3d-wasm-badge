// Browser tests for the web host. Driven from test.html via window.runTests().
(function () {
  const tests = [];
  function test(name, fn) { tests.push({ name, fn }); }

  function fbHash() {
    const fb = window.fri3d.readFb();
    let h = 2166136261 >>> 0;
    for (let i = 0; i < fb.length; i++) h = Math.imul(h ^ fb[i], 16777619) >>> 0;
    return h.toString(16).padStart(8, '0');
  }
  function assertEq(a, b, msg) {
    if (a !== b) throw new Error(`${msg ?? 'assertion'}: expected ${JSON.stringify(b)}, got ${JSON.stringify(a)}`);
  }
  function assertDistinct(values, msg) {
    if (new Set(values).size !== values.length) throw new Error(`${msg ?? 'expected distinct'}: ${values.join(' ')}`);
  }
  function appIndex(name) {
    const f = window.fri3d;
    for (let i = 0; i < f.appCount(); i++) if (f.appName(i) === name) return i;
    throw new Error(`no app named ${name}`);
  }
  function home() { window.fri3d.exitToLauncher(); window.fri3d.render(); }

  let launcherHash = null;

  test('launcher boots and renders a non-blank frame', () => {
    home();
    const fb = window.fri3d.readFb();
    if (!fb.some(p => p !== fb[0])) throw new Error('framebuffer is blank');
    launcherHash = fbHash();
  });

  test('circles: pressing OK produces a distinct frame each time', () => {
    const f = window.fri3d;
    f.rngSeed(42);
    f.startApp(appIndex('Circles'));
    f.render();
    const hashes = [];
    for (let i = 0; i < 6; i++) { f.tap(f.KEY.OK); f.render(); hashes.push(fbHash()); }
    assertDistinct(hashes, 'OK-press cycle');
    home();
  });

  test('MT19937: first value from seed(42) is canonical', () => {
    window.fri3d.rngSeed(42);
    assertEq(window.fri3d.rngGet() >>> 0, 1608637542, 'MT19937(seed=42) first get');
  });

  test('menu tap inside an app returns to the launcher', () => {
    const f = window.fri3d;
    f.startApp(appIndex('Circles'));
    f.render();
    if (fbHash() === launcherHash) throw new Error('app did not take the screen');
    f.tap(f.KEY.MENU);
    f.render();
    assertEq(fbHash(), launcherHash, 'back on launcher');
  });

  test('snake: renders a frame different from the launcher', () => {
    const f = window.fri3d;
    f.startApp(appIndex('Snake'));
    f.render(); f.advance(300); f.render(); f.advance(300); f.render();
    if (fbHash() === launcherHash) throw new Error('snake frame equals launcher frame');
    home();
  });

  test('settings app starts without a kernel error', () => {
    const f = window.fri3d;
    f.startApp(appIndex('Settings'));
    f.render();
    assertEq(f.lastError(), '', 'last_error');
    home();
  });

  test('wifi: auto-connect via the simulated radio shows the launcher icon', () => {
    const f = window.fri3d;
    // localStorage may carry a saved network from an earlier run: start
    // from a known state.
    f.wifiSetEnabled(false);
    home();
    const before = fbHash();
    if (!f.wifiSave('Fri3d Camp', 'fri3d2026')) throw new Error('wifi_save refused');
    f.wifiSetEnabled(true);
    for (let i = 0; i < 10; i++) f.advance(500);
    assertEq(f.wifiStatus(), 3, 'status (3 = connected)');
    assertEq(f.wifiSsid(), 'Fri3d Camp', 'ssid');
    f.render();
    if (fbHash() === before) throw new Error('launcher frame unchanged after connect');
  });

  window.runTests = function runTests() {
    const results = [];
    for (const t of tests) {
      try { t.fn(); results.push({ name: t.name, ok: true }); }
      catch (e) { results.push({ name: t.name, ok: false, error: String(e.message || e) }); }
    }
    const passed = results.filter(r => r.ok).length;
    return { passed, failed: results.length - passed, results };
  };
})();
