import { Context } from './Provider';

import Canvas from './Canvas';

class Frame extends Canvas {
  static contextType = Context;

  componentDidMount() {
    super.componentDidMount();
    this.connectWS();
  }

  componentWillUnmount() {
    super.componentWillUnmount();
    this.disconnectWS();
  }

  componentDidUpdate(prev) {
    if (prev.url !== this.props.url) {
      this.disconnectWS();
      this.connectWS();
    }
  }

  connectWS = () => {
    this.ws = new WebSocket(this.props.url);
    this.ws.binaryType = 'arraybuffer';

    this.ws.onmessage = (event) => {
      const buf = new Uint8Array(event.data);
      this.load(buf);
      this.context.updateError(this.props.url, false);
    };

    this.ws.onerror = () => {
      this.context.updateError(this.props.url, true);
    };
  };

  disconnectWS = () => {
    if (!this.ws) return;

    this.ws.onclose = null;
    this.ws.onerror = null;
    this.ws.onmessage = null;

    this.ws.close();
    this.ws = null;
  };

  load = (buf) => {
    const { server } = this.context.config;

    const { scale } = server;
    const [w, h] = server.res;

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
  };

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
