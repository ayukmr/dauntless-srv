import React, { Component } from 'react';

class Canvas extends Component {
  canvasRef = React.createRef();

  canvas = null;
  ctx = null;

  componentDidMount() {
    this.canvas = this.canvasRef.current;
    this.ctx = this.canvas.getContext('2d');

    this.animate();
  }

  componentWillUnmount() {
    cancelAnimationFrame(this.frameID);
  }

  animate = () => {
    this.draw();
    this.frameID = requestAnimationFrame(() => this.animate());
  };

  render() {
    return <canvas ref={this.canvasRef}></canvas>;
  }

  color = (c, id) => {
    return `${c}${id === null ? '33' : 'ff'}`;
  };

  point = (x, y, color) => {
    this.ctx.fillStyle = color;
    this.ctx.beginPath();
    this.ctx.arc(x, y, 8, 0, Math.PI * 2);
    this.ctx.fill();
  };

  line = (p0, p1, color) => {
    this.ctx.strokeStyle = color;
    this.ctx.lineWidth = 8;

    this.ctx.beginPath();
    this.ctx.moveTo(p0[0], p0[1]);
    this.ctx.lineTo(p1[0], p1[1]);
    this.ctx.stroke();
  };

  rect = (x, y, w, h, color) => {
    this.ctx.fillStyle = color;
    this.ctx.fillRect(x, y, w, h);
  };

  text = (text, x, y, color) => {
    this.ctx.font = '35px IBM Plex Sans';
    this.ctx.fillStyle = color;
    this.ctx.textAlign = 'center';
    this.ctx.textBaseline = 'middle';

    this.ctx.fillText(text, x, y);
  };
}

export default Canvas;
