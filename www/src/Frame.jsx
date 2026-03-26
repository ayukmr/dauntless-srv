import { Context } from './Provider';

import Canvas from './Canvas';

class Frame extends Canvas {
  static contextType = Context;

  componentDidMount() {
    super.componentDidMount();

    this.interval = setInterval(
      async () => {
        try {
          await this.fetch();
          this.context.error(this.props.url, false);
        } catch {
          this.context.error(this.props.url, true);
        }
      },
      50
    );
  }

  componentWillUnmount() {
    super.componentWillUnmount();
    clearInterval(this.interval);
  }

  fetch = async () => {
    const res = await fetch(this.props.url);

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

    this.canvas.width = w * scale;
    this.canvas.height = h * scale;

    this.ctx.imageSmoothingEnabled = false;

    this.ctx.putImageData(img, 0, 0);
    this.ctx.drawImage(this.canvas, 0, 0, w, h, 0, 0, w * scale, h * scale);
  }

  draw = () => {
    for (const tag of this.context.data.tags) {
      const { id, corners } = tag;

      const clr = this.color('#ff0000', id);
      const [tl, tr, bl, br] = corners;

      this.line(tl, tr, clr);
      this.line(tr, br, clr);
      this.line(br, bl, clr);
      this.line(bl, tl, clr);

      for (const corner of corners) {
        this.point(corner[0], corner[1], clr);
      }

      if (this.props.showIDs && id !== null) {
        const xs = corners.map((pt) => pt[0]);
        const ys = corners.map((pt) => pt[1]);

        const x = (Math.min(...xs) + Math.max(...xs)) / 2;
        const y = (Math.min(...ys) + Math.max(...ys)) / 2;

        this.text(id, x, y, clr);
      }
    }
  };
}

export default Frame;
