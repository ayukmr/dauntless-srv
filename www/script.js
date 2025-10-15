const fCnv = document.querySelector('#frame');
const fCtx = fCnv.getContext('2d');

const tCnv = document.querySelector('#top');
const tCtx = tCnv.getContext('2d');

const header = document.querySelector('header');

let frame = null;
let tags = null;

const errs = { frame: false, tags: false };

function draw() {
  if (frame !== null && tags !== null) {
    drawFrame();
    drawTags();
  }

  header.classList.toggle('dcnn', errs.frame && errs.tags);

  requestAnimationFrame(draw);
}

function drawFrame() {
  fCtx.putImageData(frame, 0, 0);

  for (const tag of tags) {
    const { id, corners  } = tag;

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
      createFrame(data);
      errs.frame = false;
    })
    .catch((err) => {
      console.error(err);
      errs.frame = true;
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

function createFrame(data) {
  const h = data.length;
  const w = data[0].length;

  fCnv.width = w;
  fCnv.height = h;

  frame = fCtx.createImageData(w, h);
  const pixels = frame.data;

  for (let y = 0; y < h; y++) {
    for (let x = 0; x < w; x++) {
      const l = data[y][x] * 255;
      const i = (y * w + x) * 4;

      pixels[i] = l;
      pixels[i + 1] = l;
      pixels[i + 2] = l;
      pixels[i + 3] = 255;
    }
  }
}

setInterval(update, 10);
requestAnimationFrame(draw);
