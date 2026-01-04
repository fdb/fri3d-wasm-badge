const { chromium } = require('playwright');
const { spawn } = require('child_process');
const fs = require('fs');
const path = require('path');

const rootDir = path.resolve(__dirname, '..', '..');
const webDir = path.join(rootDir, 'build', 'web');
const port = 8123;

function ensureWebBuild() {
  const indexPath = path.join(webDir, 'index.html');
  if (!fs.existsSync(indexPath)) {
    throw new Error(`Missing ${indexPath}. Run ./build_web.sh first.`);
  }
}

function startServer() {
  const server = spawn('python3', ['-m', 'http.server', `${port}`], {
    cwd: webDir,
    stdio: 'ignore',
  });
  const stop = () => {
    if (!server.killed) {
      server.kill();
    }
  };
  process.on('exit', stop);
  process.on('SIGINT', () => {
    stop();
    process.exit(1);
  });
  process.on('SIGTERM', () => {
    stop();
    process.exit(1);
  });
  return stop;
}

async function getBlackPixelCount(page) {
  return page.evaluate(() => {
    const canvas = document.getElementById('canvas');
    if (!canvas) {
      return 0;
    }
    const ctx = canvas.getContext('2d');
    if (!ctx) {
      return 0;
    }
    const data = ctx.getImageData(0, 0, canvas.width, canvas.height).data;
    let black = 0;
    for (let i = 0; i < data.length; i += 4) {
      if (data[i] === 0 && data[i + 1] === 0 && data[i + 2] === 0) {
        black += 1;
      }
    }
    return black;
  });
}

async function run() {
  ensureWebBuild();
  const stopServer = startServer();
  await new Promise((resolve) => setTimeout(resolve, 500));

  const browser = await chromium.launch();
  const page = await browser.newPage();
  try {
    await page.goto(`http://localhost:${port}/index.html`, { waitUntil: 'load' });
    await page.waitForFunction(() => {
      const canvas = document.getElementById('canvas');
      return canvas && !canvas.classList.contains('hidden');
    });

    const initialBlack = await getBlackPixelCount(page);
    if (initialBlack === 0) {
      throw new Error('Launcher did not render any black pixels.');
    }

    await page.keyboard.press('Enter');

    await page.waitForFunction(
      (baseline) => {
        const canvas = document.getElementById('canvas');
        if (!canvas) {
          return false;
        }
        const ctx = canvas.getContext('2d');
        if (!ctx) {
          return false;
        }
        const data = ctx.getImageData(0, 0, canvas.width, canvas.height).data;
        let black = 0;
        for (let i = 0; i < data.length; i += 4) {
          if (data[i] === 0 && data[i + 1] === 0 && data[i + 2] === 0) {
            black += 1;
          }
        }
        return Math.abs(black - baseline) > 50;
      },
      initialBlack,
      { timeout: 10000 }
    );
  } finally {
    await browser.close();
    stopServer();
  }
}

run().catch((err) => {
  console.error(err);
  process.exit(1);
});
