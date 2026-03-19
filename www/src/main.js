import { query } from './utils.js';
import { draw, fetchDraw } from './draw.js';
import { initConfig } from './config.js';

const fCnv = query('#frame');
const fCtx = fCnv.getContext('2d');

const mCnv = query('#mask');
const mCtx = mCnv.getContext('2d');

const fps = query('#fps');

let tags = null;
const errs = { data: false, frame: false, mask: false, corners: false };

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
          fps.innerText = Math.trunc(1000 / data.ms);
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

      fetchDraw('/api/mask', mCnv, mCtx)
        .then(() => errs.mask = false)
        .catch((err) => {
          console.error(err);
          errs.mask = true;
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
