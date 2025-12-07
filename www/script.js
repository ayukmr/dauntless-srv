const fCnv = document.querySelector('#frame');
const fCtx = fCnv.getContext('2d');

const mCnv = document.querySelector('#mask');
const mCtx = mCnv.getContext('2d');

const tCnv = document.querySelector('#top');
const tCtx = tCnv.getContext('2d');

const header = document.querySelector('header');

const settings = document.querySelectorAll('#settings input');
const reset = document.querySelector('#reset');

let config = null;

let frame = null;
let mask = null;
let tags = null;

const errs = { frame: false, mask: false, tags: false };

function draw() {
  if (frame !== null && tags !== null) {
    drawFrame();
    drawMask();
    drawTags();
  }

  header.classList.toggle('dcnn', errs.frame && errs.tags);

  requestAnimationFrame(draw);
}

function drawFrame() {
  fCtx.putImageData(frame, 0, 0);

  for (const tag of tags) {
    const { id, corners } = tag;

    if (id === null) continue;

    const [tl, tr, bl, br] = corners;

    const xs = corners.map((pt) => pt[0]);
    const ys = corners.map((pt) => pt[1]);

    const x = (Math.min(...xs) + Math.max(...xs)) / 2;
    const y = (Math.min(...ys) + Math.max(...ys)) / 2;

    line(fCtx, tl, tr, 'red');
    line(fCtx, tr, br, 'red');
    line(fCtx, br, bl, 'red');
    line(fCtx, bl, tl, 'red');

    text(fCtx, id, x, y);
  }
}

function drawMask() {
  mCtx.putImageData(mask, 0, 0);

  for (const tag of tags) {
    const { id, corners } = tag;

    if (id === null) continue;

    for (const corner of corners) {
      point(mCtx, corner[0], corner[1], 'red');
      point(mCtx, corner[0], corner[1], 'red');
      point(mCtx, corner[0], corner[1], 'red');
      point(mCtx, corner[0], corner[1], 'red');
    }
  }
}

function drawTags() {
  tCnv.width = fCnv.height;
  tCnv.height = fCnv.height;

  const w = tCnv.width;
  const h = tCnv.height;

  rect(tCtx, 0, 0, w, h, 'black');
  rect(tCtx, w/2 - 50, h - 25, 100, 50, 'red');

  for (const tag of tags) {
    const { id, rot, pos } = tag;

    if (id === null) continue;

    const x = w/2 + pos[0] * 50;
    const y = h - 25 - pos[2] * 50;

    const dx = Math.cos(-rot) * 50;
    const dy = Math.sin(-rot) * 50;

    const p0 = [x - dx/2, y - dy/2];
    const p1 = [x + dx/2, y + dy/2];

    line(tCtx, p0, p1, 'blue');
    line(tCtx, [w/2, h - 25], [x, y], 'red');

    text(tCtx, `${id}@${Math.round(rot * 180/Math.PI)}°`, x, y);
  }
}

function rect(ctx, x, y, w, h, color) {
  ctx.fillStyle = color;
  ctx.fillRect(x, y, w, h);
}

function point(ctx, x, y, color) {
  ctx.fillStyle = color;
  ctx.beginPath();
  ctx.arc(x, y, 4, 0, Math.PI * 2);
  ctx.fill();
}

function line(ctx, p0, p1, color) {
  ctx.strokeStyle = color;
  ctx.lineWidth = 3;

  ctx.beginPath();
  ctx.moveTo(p0[0], p0[1]);
  ctx.lineTo(p1[0], p1[1]);
  ctx.stroke();
}

function text(ctx, text, x, y) {
  ctx.font = '14px Menlo';
  ctx.fillStyle = 'red';
  ctx.textAlign = 'center';
  ctx.textBaseline = 'middle';

  ctx.fillText(text, x, y);
}

function update() {
  fetch('/api/frame')
    .then((res) => res.json())
    .then((data) => {
      frame = createFrame(data, fCnv, fCtx);
      errs.frame = false;
    })
    .catch((err) => {
      console.error(err);
      errs.frame = true;
    });

  fetch('/api/mask')
    .then((res) => res.json())
    .then((data) => {
      mask = createFrame(data, mCnv, mCtx);
      errs.mask = false;
    })
    .catch((err) => {
      console.error(err);
      errs.mask = true;
    });

  fetch('/api/tags')
    .then((res) => res.json())
    .then((data) => {
      tags = data;
      errs.tags = false;
    })
    .catch((err) => {
      console.error(err);
      errs.tags = true;
    });
}

function createFrame(data, cnv, ctx) {
  const h = data.length;
  const w = data[0].length;

  const scale = 2;

  cnv.width = w * scale;
  cnv.height = h * scale;

  const frame = ctx.createImageData(w * scale, h * scale);
  const pixels = frame.data;

  for (let y = 0; y < h; y++) {
    for (let x = 0; x < w; x++) {
      const l = data[y][x] * 255;

      for (let dy = 0; dy < scale; dy++) {
        for (let dx = 0; dx < scale; dx++) {
          const px = x * scale + dx;
          const py = y * scale + dy;

          const i = (py * w * scale + px) * 4;

          pixels[i] = l;
          pixels[i + 1] = l;
          pixels[i + 2] = l;
          pixels[i + 3] = 255;
        }
      }
    }
  }

  return frame;
}

function initSettings() {
  fetch('/api/config')
    .then((res) => res.json())
    .then((data) => {
      config = data;

      settings.forEach((input) => {
        input.value = config[input.name];
        input.addEventListener('input', () => setConfig());
      });

      reset.addEventListener('click', () => resetConfig());
    });
}

function setConfig() {
  settings.forEach((input) => {
    config[input.name] = parseFloat(input.value);
  });

  fetch('/api/config', {
    method: 'POST',
    body: JSON.stringify(config)
  });
}

function resetConfig() {
  fetch('/api/config/reset', { method: 'POST' })
    .then((res) => res.json())
    .then((data) => {
      config = data;

      settings.forEach((input) => {
        input.value = config[input.name];
        input.addEventListener('input', () => setConfig());
      });
    });
}

initSettings();

setInterval(update, 10);
requestAnimationFrame(draw);
