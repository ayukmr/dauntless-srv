import { query } from './utils.js';
import { draw, fetchDraw } from './draw.js';
import { initConfig } from './config.js';

const fCnv = query('#frame');
const fCtx = fCnv.getContext('2d');
const mCnv = query('#mask');
const mCtx = mCnv.getContext('2d');
const ms = query('#ms');

let tags = null;
const errs = { frame: false, mask: false, tags: false };

function update() {
  fetch('/api/ms')
    .then((res) => res.json())
    .then((f) => {
      ms.innerText = f.toFixed(2);
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

initConfig();

setInterval(update, 50);
requestAnimationFrame(() => draw(() => tags, errs));
