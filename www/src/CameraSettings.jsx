import React, { Component } from 'react';
import { Context } from './Provider';

import Popup from './Popup';

class CameraSettings extends Component {
  static contextType = Context;

  sliderRef = React.createRef();

  render() {
    const { meta, config: { detector, server }, update } = this.context;

    const camRes = meta.cams[server.camera][1];
    const curRes = camRes[Math.floor(this.sliderRef.current?.value / 100 * 0.99 * camRes.length)];

    return <Popup
      header="Camera"
      show={this.props.show}
      onCancel={this.props.onCancel}
    >
      <div>
        <label htmlFor="camera">Camera</label>
        <select
          style={{ width: 250 }}
          value={server.camera}
          onChange={(e) => {
            const camera = +e.target.value;
            update('server', { camera, res: meta.cams[camera][1][0] });
          }}
        >
          {Object.entries(this.context.meta.cams).map(([i, cam]) => (
            <option key={`cam${i}`} value={i}>{cam[0]}</option>
          ))}
        </select>
      </div>

      <div>
        <label htmlFor="fov">FOV</label>
        <input
          type="number"
          name="fov"
          defaultValue={detector.fov}
          onChange={(e) => update('detector', { fov: +e.target.value })}
        />
      </div>

      <div>
        <label htmlFor="res">Resolution</label>
        <div style={{ display: 'flex', flexDirection: 'column', width: 250 }}>
          <input
            style={{ marginBottom: 0 }}
            type="range"
            name="res"
            ref={this.sliderRef}
            value={
              (camRes.findIndex(
                ([w, h]) => w === server.res[0] && h === server.res[1]
              ) + 0.5) / camRes.length * 100
            }
            onChange={() => update('server', { res: curRes })}
          />
          <span style={{ textAlign: 'center', marginBottom: 12 }}>
            {curRes?.[0]}×{curRes?.[1]}
          </span>
        </div>
      </div>

      <div>
        <label htmlFor="scale">Preview Scale</label>
        <input
          type="number"
          name="scale"
          defaultValue={server.scale}
          onChange={(e) => update('server', { scale: +e.target.value })}
        />
      </div>
    </Popup>;
  }
}

export default CameraSettings;
