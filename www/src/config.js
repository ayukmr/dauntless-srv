import { query, queryAll } from './utils.js';

const settings = queryAll('#settings input');
const reset = query('#reset');

let config = null;

function initConfig() {
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

export { initConfig };
