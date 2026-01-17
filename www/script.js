const SCALE = 4;

const query = (q) => document.querySelector(q);
const queryAll = (q) => document.querySelectorAll(q);

const fCnv = query('#frame');
const fCtx = fCnv.getContext('2d');
const mCnv = query('#mask');
const mCtx = mCnv.getContext('2d');
const tCnv = query('#top');
const tCtx = tCnv.getContext('2d');

const fps = query('#fps');
const header = query('header');
const settings = queryAll('#settings input');
const reset = query('#reset');

let config = null;
let tags = null;

const errs = { frame: false, tags: false };

function draw() {
  if (tags !== null) {
    drawFrame();
    drawMask();
    drawTags();
  }

  header.classList.toggle('dcnn', errs.frame && errs.tags);

  requestAnimationFrame(draw);
}

function drawFrame() {
  for (const tag of tags) {
    const { id, corners } = tag;

    const [tl, tr, bl, br] = corners;

    const xs = corners.map((pt) => pt[0]);
    const ys = corners.map((pt) => pt[1]);

    const x = (Math.min(...xs) + Math.max(...xs)) / 2;
    const y = (Math.min(...ys) + Math.max(...ys)) / 2;

    const clr = color('#ff0000', id);

    line(fCtx, tl, tr, clr);
    line(fCtx, tr, br, clr);
    line(fCtx, br, bl, clr);
    line(fCtx, bl, tl, clr);

    if (id !== null) {
      text(fCtx, id, x, y);
    }
  }
}

function drawMask() {
  for (const tag of tags) {
    const { id, corners } = tag;

    const clr = color('#ff0000', id);
    for (const corner of corners) {
      point(mCtx, corner[0], corner[1], clr);
      point(mCtx, corner[0], corner[1], clr);
      point(mCtx, corner[0], corner[1], clr);
      point(mCtx, corner[0], corner[1], clr);
    }
  }
}

function drawTags() {
  tCnv.width = fCnv.height;
  tCnv.height = fCnv.height;

  const w = tCnv.width;
  const h = tCnv.height;

  rect(tCtx, 0, 0, w, h, '#000000');
  rect(tCtx, w/2 - 50, h - 25, 100, 50, '#333333');

  for (const tag of tags) {
    const { id, rot, pos } = tag;

    const x = w/2 + pos[0] * 50;
    const y = h - 25 - pos[2] * 50;

    const dx = Math.cos(-rot) * 50;
    const dy = Math.sin(-rot) * 50;

    const p0 = [x - dx/2, y - dy/2];
    const p1 = [x + dx/2, y + dy/2];

    line(tCtx, p0, p1, color('#0000ff', id));
    line(tCtx, [w/2, h - 25], [x, y], color('#ff0000', id));

    if (id !== null) {
      text(tCtx, `${id}@${Math.round(rot * 180/Math.PI)}°`, x, y);
    }
  }
}

function color(c, id) {
  return `${c}${id === null ? '55' : 'ff'}`;
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
  ctx.fillStyle = '#ff0000';
  ctx.textAlign = 'center';
  ctx.textBaseline = 'middle';

  ctx.fillText(text, x, y);
}

function update() {
  fetch('/api/fps')
    .then((res) => res.json())
    .then((buf) => {
      fps.innerText = buf.toFixed(2);
    });

  fetchDraw('/api/frame', fCnv, fCtx)
    .then(() => errs.frame = false)
    .catch((err) => {
      console.error(err);
      errs.frame = true;
    });

  fetchDraw('/api/mask', mCnv, mCtx)
    .then(() => errs.mask = false)
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

async function fetchDraw(url, cnv, ctx) {
  const blob = await fetch(url).then((r) => r.blob());
  const bmp = await createImageBitmap(blob);

  cnv.width = bmp.width * SCALE;
  cnv.height = bmp.height * SCALE;

  ctx.imageSmoothingEnabled = false;

  ctx.drawImage(bmp, 0, 0);
  ctx.drawImage(cnv, 0, 0, bmp.width, bmp.height, 0, 0, cnv.width, cnv.height);
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
