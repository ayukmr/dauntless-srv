import { query } from './utils.js';
import { color, point, line, rect, text } from './canvas.js';

const fCnv = query('#frame');
const fCtx = fCnv.getContext('2d');
const mCnv = query('#mask');
const mCtx = mCnv.getContext('2d');

const tCnv = query('#tags');
const tCtx = tCnv.getContext('2d');

const header = query('header');
const detections = query('#detections');

function draw(getTags, errs) {
  const tags = getTags();

  if (tags !== null) {
    drawFrame(tags);
    drawMask(tags);
    drawTags(tags);
    listDetections(tags);
  }

  header.classList.toggle('dcnn', errs.frame || errs.mask || errs.tags);

  requestAnimationFrame(() => draw(getTags, errs));
}

function drawFrame(tags) {
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

function drawMask(tags) {
  for (const tag of tags) {
    const { id, corners } = tag;

    const clr = color('#ff0000', id);

    for (const corner of corners) {
      point(mCtx, corner[0], corner[1], clr);
    }
  }
}

function drawTags(tags) {
  tCnv.width = fCnv.height;
  tCnv.height = fCnv.height;

  const w = tCnv.width;
  const h = tCnv.height;

  rect(tCtx, 0, 0, w, h, '#000000');
  rect(tCtx, w/2 - 50, h - 25, 100, 50, '#333333');

  for (const tag of tags) {
    const { id, rot, pos } = tag;

    if (id === null) continue;

    const x = w/2 + pos[0] * 100;
    const y = h - 25 - pos[2] * 100;

    const dx = Math.cos(-rot) * 50;
    const dy = Math.sin(-rot) * 50;

    const p0 = [x - dx/2, y - dy/2];
    const p1 = [x + dx/2, y + dy/2];

    line(tCtx, p0, p1, '#0000ff');
    line(tCtx, [w/2, h - 25], [x, y], '#ff0000');

    text(tCtx, `${id}@${Math.round(rot * 180/Math.PI)}°`, x, y);
  }
}

function listDetections(tags) {
  detections.innerText = '';

  tags
    .filter((tag) => tag.id !== null)
    .forEach((tag, i) => {
      if (i !== 0) {
        detections.appendChild(document.createElement('hr'));
      }

      const { id, rot, pos } = tag;

      const el = document.createElement('div');

      const h = document.createElement('h4');
      h.innerText = `Tag ${id}`;
      const p = document.createElement('p');
      p.innerText = `Rot: ${rot.toFixed(2)}\nX: ${pos[0].toFixed(2)}, Y: ${pos[1].toFixed(2)}, Z: ${pos[2].toFixed(2)}`;

      el.appendChild(h);
      el.appendChild(p);
      detections.appendChild(el);
    });
}

async function fetchDraw(url, cnv, ctx) {
  const res = await fetch(url);

  const w = +res.headers.get('X-Width');
  const h = +res.headers.get('X-Height');
  const scale = +res.headers.get('X-Scale');

  const buf = new Uint8Array(await res.arrayBuffer());
  const img = new ImageData(w, h);

  for (let i = 0; i < buf.length; i++) {
    const v = buf[i];
    img.data[i * 4 + 0] = v;
    img.data[i * 4 + 1] = v;
    img.data[i * 4 + 2] = v;
    img.data[i * 4 + 3] = 255;
  }

  cnv.width = w * scale;
  cnv.height = h * scale;

  ctx.imageSmoothingEnabled = false;

  ctx.putImageData(img, 0, 0);
  ctx.drawImage(cnv, 0, 0, w, h, 0, 0, cnv.width, cnv.height);
}

export { draw, fetchDraw };
