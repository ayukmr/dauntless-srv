import Canvas from './Canvas';
import { Context } from './Provider';

class Tags extends Canvas {
  static contextType = Context;

  draw = () => {
    this.canvas.width = 600;
    this.canvas.height = 600;

    const w = this.canvas.width;
    const h = this.canvas.height;

    this.rect(0, 0, w, h, '#000000');
    this.rect(w/2 - 50, h - 25, 100, 50, '#333333');

    for (const tag of this.context.data.tags) {
      const { id, rot, pos } = tag;

      if (id === null) continue;

      const x = w/2 + pos[0] * 100;
      const y = h - 25 - pos[2] * 100;

      const dx = Math.cos(rot) * 75;
      const dy = Math.sin(rot) * 75;

      const p0 = [x - dx/2, y - dy/2];
      const p1 = [x + dx/2, y + dy/2];

      this.line(p0, p1, '#0000ff');
      this.line([w/2, h - 25], [x, y], '#ff0000');

      this.text(id, x, y, '#00ff00');
    }
  }
}

export default Tags;
