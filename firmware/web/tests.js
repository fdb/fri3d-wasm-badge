// Browser-side tests for the Fri3d firmware. Loaded into the harness
// via test.html and driven from Playwright with window.runTests().
//
// Each test is a small function that throws on failure. runTests() catches
// those and returns a {passed, failed} summary, suitable for assertion in
// Playwright via browser_evaluate.

(function () {
  const tests = [];
  function test(name, fn) { tests.push({ name, fn }); }

  // ---- Helpers ------------------------------------------------------------

  function fbHash() {
    const fb = window.fri3d.readFb();
    let h = 2166136261 >>> 0;
    for (let i = 0; i < fb.length; i++) {
      h = Math.imul(h ^ fb[i], 16777619) >>> 0;
    }
    return h.toString(16).padStart(8, '0');
  }

  function assertEq(actual, expected, msg) {
    if (actual !== expected) {
      throw new Error(`${msg ?? 'assertion'}: expected ${JSON.stringify(expected)}, got ${JSON.stringify(actual)}`);
    }
  }

  function assertDistinct(values, msg) {
    const set = new Set(values);
    if (set.size !== values.length) {
      throw new Error(`${msg ?? 'expected distinct'}: ${JSON.stringify(values)} has duplicates (${set.size}/${values.length} unique)`);
    }
  }

  // ---- Tests --------------------------------------------------------------

  test('circles app: pressing A produces a distinct frame each time', () => {
    // Reset by re-seeding the host RNG — keeps this test deterministic
    // and independent of prior test runs.
    window.fri3d.rngSeed(42);
    window.fri3d.render();
    const hashes = [];
    for (let i = 0; i < 6; i++) {
      window.fri3d.tap(window.fri3d.KEY.OK);
      window.fri3d.render();
      hashes.push(fbHash());
    }
    // A correct reshuffle yields 6 distinct frames; a stuck SEED yields at
    // most 2 unique hashes (a 1- or 2-cycle).
    assertDistinct(hashes, `A-press cycle: ${hashes.join(' ')}`);
  });

  test('MT19937: first value from seed(42) matches canonical output', () => {
    // MT19937 seeded with 42 gives a known first output. Guards us against
    // accidental breakage of the PRNG port in random.cpp.
    window.fri3d.rngSeed(42);
    const first = window.fri3d.rngGet() >>> 0;
    assertEq(first, 1608637542, 'MT19937(seed=42) first get');
  });

  test('MT19937: seed(0) and seed(0) again yield the same sequence', () => {
    window.fri3d.rngSeed(0);
    const a = [window.fri3d.rngGet(), window.fri3d.rngGet(), window.fri3d.rngGet()];
    window.fri3d.rngSeed(0);
    const b = [window.fri3d.rngGet(), window.fri3d.rngGet(), window.fri3d.rngGet()];
    assertEq(JSON.stringify(a), JSON.stringify(b), 'replayable from same seed');
  });

  test('app-switching: OK on launcher swaps to circles, Back returns', () => {
    // Baseline: the launcher frame contains distinctive pixels (e.g. text).
    window.fri3d.render();
    const launcherHash = fbHash();

    // Tap OK while "Circles" is highlighted — which it is on boot, since the
    // launcher's MENU_SCROLL static starts at 0.
    window.fri3d.tap(window.fri3d.KEY.OK);
    window.fri3d.render();
    const circlesHash = fbHash();
    if (circlesHash === launcherHash) {
      throw new Error('start_app(1) did not change the frame — module swap failed');
    }

    // Long-press Back to return to launcher.
    window.fri3d.tap(window.fri3d.KEY.BACK, 500);
    window.fri3d.render();
    const backHash = fbHash();
    if (backHash === circlesHash) {
      throw new Error('exit_to_launcher did not change the frame — back-swap failed');
    }
  });

  // ---- Runner -------------------------------------------------------------

  window.runTests = function runTests() {
    const results = [];
    for (const t of tests) {
      try {
        t.fn();
        results.push({ name: t.name, ok: true });
      } catch (e) {
        results.push({ name: t.name, ok: false, error: String(e.message || e) });
      }
    }
    const passed = results.filter(r => r.ok).length;
    const failed = results.length - passed;
    return { passed, failed, results };
  };
})();
