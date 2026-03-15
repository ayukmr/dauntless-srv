import { query } from './utils.js';
import { draw, fetchDraw } from './draw.js';
import { initConfig } from './config.js';

const fCnv = query('#frame');
const fCtx = fCnv.getContext('2d');

const eCnv = query('#edges');
const eCtx = eCnv.getContext('2d');
const cCnv = query('#corners');
const cCtx = cCnv.getContext('2d');

const ms = query('#ms');

let tags = null;
const errs = { data: false, frame: false, edges: false, corners: false };

let updating = false;

async function update() {
  if (updating) return;
  updating = true;

  try {
    const promises = [
      fetch('/api/data')
        .then((res) => res.json())
        .then((data) => {
          tags = data.tags;
          ms.innerText = data.ms.toFixed(2);
          errs.data = false;
        })
        .catch((err) => {
          console.error(err);
          errs.data = true;
        }),

      fetchDraw('/api/frame', fCnv, fCtx)
        .then(() => errs.frame = false)
        .catch((err) => {
          console.error(err);
          errs.frame = true;
        }),

      fetchDraw('/api/edges', eCnv, eCtx)
        .then(() => errs.edges = false)
        .catch((err) => {
          console.error(err);
          errs.edges = true;
        }),

      fetchDraw('/api/corners', cCnv, cCtx, true)
        .then(() => errs.corners = false)
        .catch((err) => {
          console.error(err);
          errs.corners = true;
        }),
    ];

    await Promise.all(promises);
  } finally {
    updating = false;
  }
}

initConfig();

setInterval(update, 50);
requestAnimationFrame(() => draw(() => tags, errs));
